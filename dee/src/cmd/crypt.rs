use std::{fs, io};

use anyhow::{anyhow, Result};
use drand_core::{
    chain::{self, ChainClient, ChainOptions},
    http_chain_client::HttpChainClient,
};

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

    if info.is_signature_on_g1() {
        return Err(anyhow!("remote must have signatures on G2"));
    }
    if !info.is_unchained() {
        return Err(anyhow!("remote must use unchained signatures"));
    }

    let beacon_time = crate::time::round_from_option(chain, round).await?;

    let src = file_or_stdin(input)?;
    let dst = file_or_stdout(output)?;

    tlock_age::encrypt(
        dst,
        src,
        armor,
        &info.hash(),
        &info.public_key(),
        beacon_time.round(),
    )
    .map(|()| String::from(""))
}

pub async fn decrypt(
    _cfg: &config::Local,
    output: Option<String>,
    input: Option<String>,
    chain: ConfigChain,
) -> Result<String> {
    // todo(thibault): make this work with stdin
    let src = file_or_stdin(input.clone())?;
    let header = tlock_age::decrypt_header(src)?;

    let info = chain.info();
    let chain = chain::Chain::new(&chain.url());

    let client = HttpChainClient::new(
        chain,
        Some(ChainOptions::new(true, true, Some(info.clone().into()))),
    );

    let time = RandomnessBeaconTime::from_round(&info, header.round());

    let beacon = match client.get(header.round()).await {
        Ok(beacon) => beacon,
        Err(_) => {
            let relative = time.relative();
            let seconds = relative.num_seconds().abs() % 60;
            let minutes = (relative.num_minutes()).abs() % 60;
            let hours = relative.num_hours().abs();
            let relative = format!("{hours:0<2}:{minutes:0<2}:{seconds:0<2}");
            return Err(anyhow!(
                "Too early. Decryption round is {}, estimated in {} ({}).",
                time.round(),
                relative,
                time.absolute(),
            ));
        }
    };

    let src = file_or_stdin(input)?;
    let dst = file_or_stdout(output)?;
    tlock_age::decrypt(dst, src, &header.hash(), &beacon.signature()).map(|()| String::from(""))
}
