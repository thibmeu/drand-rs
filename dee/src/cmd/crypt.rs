use std::{cmp::Ordering, fs, io};

use anyhow::{anyhow, Result};
use colored::Colorize;
use drand_core::{
    beacon::{BeaconError, RandomnessBeaconTime},
    chain::ChainInfo,
    ChainOptions, DrandError, HttpClient,
};
use serde::Serialize;
use tlock_age::Header;

use crate::{
    config::{self, ConfigChain},
    print::{print_with_format, Format, Print},
};

pub fn file_or_stdin(input: Option<String>) -> Result<Box<dyn io::Read>> {
    let reader: Box<dyn io::Read> = match input {
        Some(path) => Box::new(io::BufReader::new(
            fs::File::open(path).map_err(|_e| anyhow!("cannot read input file"))?,
        )),
        None => Box::new(io::BufReader::new(io::stdin())),
    };
    Ok(reader)
}

pub fn file_or_stdout(output: Option<String>) -> Result<Box<dyn io::Write>> {
    let writer: Box<dyn io::Write> = match output {
        Some(path) => Box::new(io::BufWriter::new(
            fs::File::create(path).map_err(|_e| anyhow!("cannot create output file"))?,
        )),
        None => Box::new(io::BufWriter::new(io::stdout())),
    };
    Ok(writer)
}

pub fn encrypt(
    _cfg: &config::Local,
    output: Option<String>,
    input: Option<String>,
    armor: bool,
    chain: ConfigChain,
    round: Option<String>,
) -> Result<String> {
    let info = chain.info();
    if !info.is_unchained() {
        return Err(anyhow!("remote must use unchained signatures"));
    }

    let beacon_time = crate::time::round_from_option(&chain, round)?;

    let src = file_or_stdin(input)?;
    let dst = file_or_stdout(output)?;
    if armor {
        let mut dst = tlock_age::armor::ArmoredWriter::wrap_output(dst)?;
        if info.is_rfc9380() {
            tlock_age::encrypt(
                &mut dst,
                src,
                &info.hash(),
                &info.public_key(),
                beacon_time.round(),
            )?;
        } else {
            tlock_age_non_rfc9380::encrypt(
                &mut dst,
                src,
                &info.hash(),
                &info.public_key(),
                beacon_time.round(),
            )?;
        }
        dst.finish()?;
        Ok(())
    } else if info.is_rfc9380() {
        tlock_age::encrypt(
            dst,
            src,
            &info.hash(),
            &info.public_key(),
            beacon_time.round(),
        )
        .map_err(|err| anyhow!(err))
    } else {
        tlock_age_non_rfc9380::encrypt(
            dst,
            src,
            &info.hash(),
            &info.public_key(),
            beacon_time.round(),
        )
        .map_err(|err| anyhow!(err))
    }
    .map(|()| String::from(""))
    .map_err(|err| anyhow!(err))
}

pub fn inspect(
    cfg: &config::Local,
    format: Format,
    input: Option<String>,
    chain: ConfigChain,
) -> Result<String> {
    let src = file_or_stdin(input)?;
    let header = tlock_age::decrypt_header(src)?;

    let result = if let Some((name, chain_config)) = cfg.chain_by_hash(&header.hash()) {
        let is_upstream = chain.info().hash() == chain_config.info().hash();
        InspectResult::new(header, Some(name), is_upstream, Some(chain_config.info()))
    } else {
        InspectResult::new(header, None, false, None)
    };

    print_with_format(result, format)
}

pub fn decrypt(
    cfg: &config::Local,
    output: Option<String>,
    input: Option<String>,
    chain: ConfigChain,
) -> Result<String> {
    let mut src = ResetReader::new(file_or_stdin(input)?);
    let header = tlock_age::decrypt_header(&mut src)?;
    // Once headers have been read, reset the reader to pass it as if unmodified to tlock_age::decrypt
    // This allows the same reader to be used twice.
    src.reset();

    let info = chain.info();

    if header.hash() != info.hash() {
        if let Some((name, _chain_config)) = cfg.chain_by_hash(&header.hash()) {
            return Err(anyhow!("decryption failed.\nDid you forget `-u {name}`?"));
        }
    };

    let client = HttpClient::new(
        &chain.url(),
        Some(ChainOptions::new(true, true, Some(info.clone().into()))),
    )?;

    let time = RandomnessBeaconTime::from_round(&info.clone().into(), header.round());

    let beacon = match client.get(header.round()) {
        Ok(beacon) => beacon,
        Err(DrandError::Beacon(e)) => match *e {
            BeaconError::NotFound => return crate::cmd::rand::RandResult::new(None, time).short(),
            err => return Err(err.into()),
        },
        Err(e) => return Err(e.into()),
    };

    let dst = file_or_stdout(output)?;
    if info.is_rfc9380() {
        tlock_age::decrypt(dst, src, &header.hash(), &beacon.signature())
            .map(|()| String::from(""))
            .map_err(|err| anyhow!(err))
    } else {
        tlock_age_non_rfc9380::decrypt(dst, src, &header.hash(), &beacon.signature())
            .map(|()| String::from(""))
            .map_err(|err| anyhow!(err))
    }
}

#[derive(Serialize)]
struct InspectResult {
    round: u64,
    #[serde(with = "hex::serde")]
    hash: Vec<u8>,
    chain_name: Option<String>,
    is_upstream: bool,
    chain_info: Option<ChainInfo>,
}

impl InspectResult {
    pub fn new(
        header: Header,
        chain_name: Option<String>,
        is_upstream: bool,
        chain_info: Option<ChainInfo>,
    ) -> Self {
        Self {
            round: header.round(),
            hash: header.hash(),
            chain_name,
            is_upstream,
            chain_info,
        }
    }

    pub fn round(&self) -> u64 {
        self.round
    }

    pub fn hash(&self) -> Vec<u8> {
        self.hash.clone()
    }

    pub fn chain_name(&self) -> Option<String> {
        self.chain_name.clone()
    }

    pub fn chain(&self) -> Option<ChainInfo> {
        self.chain_info.clone()
    }

    pub fn is_upstream(&self) -> bool {
        self.is_upstream
    }
}

impl Print for InspectResult {
    fn short(&self) -> Result<String> {
        Ok(format!("{} {}", self.round(), hex::encode(self.hash())))
    }

    fn long(&self) -> Result<String> {
        let mut output: Vec<String> = vec![];

        // Round information
        output.push(format!("{: <11}: {}", "Round".bold(), self.round()));
        if let Some(chain) = self.chain() {
            let format =
                time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]Z")?;
            let time = RandomnessBeaconTime::new(&chain.into(), &self.round().to_string());
            let relative = time.relative();
            let seconds = relative.whole_seconds().abs() % 60;
            let minutes = (relative.whole_minutes()).abs() % 60;
            let hours = relative.whole_hours().abs();
            let epoch = match relative.whole_seconds().cmp(&0) {
                Ordering::Less => "ago",
                Ordering::Equal => "now",
                Ordering::Greater => "from now",
            };
            let relative = format!("{hours:0>2}:{minutes:0>2}:{seconds:0>2} {epoch}");
            output.push(format!("{: <11}: {}", "Relative".bold(), relative));
            output.push(format!(
                "{: <11}: {}",
                "Absolute".bold(),
                time.absolute().format(&format)?
            ));
        }

        // Hash information
        if let Some(name) = self.chain_name() {
            output.push(format!("{: <11}: {}", "Remote Name".bold(), name));
        }
        output.push(format!(
            "{: <11}: {}",
            "Upstream".bold(),
            self.is_upstream()
        ));
        output.push(format!(
            "{: <11}: {}",
            "Chain Hash".bold(),
            hex::encode(self.hash())
        ));
        Ok(output.join("\n"))
    }

    fn json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }
}

// Reader buffering every read, and with the ability to re-read what's been read already.
// This is useful when one need to use stdin reader twice.
struct ResetReader<R> {
    inner: R,
    buf: Vec<u8>,
    offset: usize,
    is_buffer_enabled: bool,
}

impl<R: std::io::Read> ResetReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buf: vec![],
            offset: 0,
            is_buffer_enabled: true,
        }
    }

    /// Reset offset, allowing read from a buf again
    pub fn reset(&mut self) {
        self.offset = 0;
        self.is_buffer_enabled = false;
    }
}

impl<R: std::io::Read> std::io::Read for ResetReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Buffer contains enough bytes for this read
        if self.buf.len() < self.offset {
            buf.copy_from_slice(&self.buf[..buf.len()]);
            self.offset += buf.len();
            return Ok(buf.len());
        }

        // Read is a mix of read from buffer and from the reader
        let n_read_bytes = self.buf.len() - self.offset;
        let (from_buf, from_read) = buf.split_at_mut(n_read_bytes);
        // First read from the inner buffer
        if n_read_bytes > 0 {
            from_buf.copy_from_slice(self.buf.as_slice());
            self.offset += n_read_bytes;
        }

        if from_read.is_empty() {
            return Ok(n_read_bytes);
        }

        // Now read from the reader
        let r = match self.inner.read(from_read) {
            Ok(size) => size,
            Err(e) => return Err(e),
        };
        if self.is_buffer_enabled {
            self.buf.append(from_read[..r].to_vec().as_mut());
            self.offset += r;
        }
        Ok(n_read_bytes + r)
    }
}
