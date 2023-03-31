use std::{fs, io};

use anyhow::{anyhow, Result};
use drand_core::{chain::ChainOptions, http_chain_client::HttpChainClient};

use crate::{
    config::{self, ConfigChain},
    time::RandomnessBeaconTime,
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

pub async fn encrypt(
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

    let beacon_time = crate::time::round_from_option(&chain, round).await?;

    let src = file_or_stdin(input)?;
    let dst = file_or_stdout(output)?;
    if armor {
        let mut dst = tlock_age::armor::ArmoredWriter::wrap_output(dst)?;
        tlock_age::encrypt(
            &mut dst,
            src,
            &info.hash(),
            &info.public_key(),
            beacon_time.round(),
        )?;
        dst.finish()?;
        Ok(())
    } else {
        tlock_age::encrypt(
            dst,
            src,
            &info.hash(),
            &info.public_key(),
            beacon_time.round(),
        )
    }
    .map(|()| String::from(""))
}

pub async fn decrypt(
    _cfg: &config::Local,
    output: Option<String>,
    input: Option<String>,
    chain: ConfigChain,
) -> Result<String> {
    let mut src = ResetReader::new(file_or_stdin(input.clone())?);
    let header = tlock_age::decrypt_header(&mut src)?;
    // Once headers have been read, reset the reader to pass it as if unmodified to tlock_age::decrypt
    // This allows the same reader to be used twice.
    src.reset();

    let info = chain.info();

    let client = HttpChainClient::new(
        &chain.url(),
        Some(ChainOptions::new(true, true, Some(info.clone().into()))),
    )?;

    let time = RandomnessBeaconTime::from_round(&info, header.round());

    let beacon = match client.get(header.round()).await {
        Ok(beacon) => beacon,
        Err(_) => {
            let relative = time.relative();
            let seconds = relative.num_seconds().abs() % 60;
            let minutes = relative.num_minutes().abs() % 60;
            let hours = relative.num_hours().abs();
            let relative = format!("{hours:0>2}:{minutes:0>2}:{seconds:0>2}");
            return Err(anyhow!(
                "Too early. Decryption round is {}, estimated in {} ({}).",
                time.round(),
                relative,
                time.absolute(),
            ));
        }
    };

    let dst = file_or_stdout(output)?;
    tlock_age::decrypt(dst, src, &header.hash(), &beacon.signature()).map(|()| String::from(""))
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
