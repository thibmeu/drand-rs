use std::{fs, io};

use anyhow::{anyhow, Result};
use drand_core::{
    chain::{self, ChainClient, ChainOptions},
    http_chain_client::HttpChainClient,
};

use crate::config::{self, ConfigChain};

pub fn file_or_stdin(input: Option<String>) -> Box<dyn io::Read> {
    let reader: Box<dyn io::Read> = match input {
        Some(path) => Box::new(io::BufReader::new(
            fs::File::open(path)
                .map_err(|_e| anyhow!("error reading input file"))
                .unwrap(),
        )),
        None => Box::new(io::BufReader::new(io::stdin())),
    };
    reader
}

pub fn file_or_stdout(output: Option<String>) -> Box<dyn io::Write> {
    let writer: Box<dyn io::Write> = match output {
        Some(path) => Box::new(io::BufWriter::new(
            fs::File::create(path)
                .map_err(|_e| anyhow!("error creating output file"))
                .unwrap(),
        )),
        None => Box::new(io::BufWriter::new(io::stdout())),
    };
    writer
}

pub async fn encrypt(
    _cfg: &config::Local,
    output: Option<String>,
    input: Option<String>,
    armor: bool,
    chain: ConfigChain,
    round: u64,
) -> Result<String> {
    let chain = chain::Chain::new(&chain.url());
    let info = chain.info().await?;

    let src = file_or_stdin(input);
    let dst = file_or_stdout(output);

    tlock_age::encrypt(dst, src, armor, &info.hash(), &info.public_key(), round)
        .map(|()| String::from(""))
}

pub async fn decrypt(
    _cfg: &config::Local,
    output: Option<String>,
    input: Option<String>,
    chain: ConfigChain,
) -> Result<String> {
    // todo(thibault): make this work with stdin
    let src = file_or_stdin(input.clone());
    let header = tlock_age::decrypt_header(src)?;

    let chain = chain::Chain::new(&chain.url());
    let info = chain.info().await?;

    let client = HttpChainClient::new(
        chain,
        Some(ChainOptions::new(true, true, Some(info.clone().into()))),
    );
    let beacon = match client.get(header.round()).await {
        Ok(beacon) => beacon,
        Err(_) => {
            return Err(anyhow!(
                "Too early. Decryption round is {}.",
                header.round()
            ))
        }
    };

    let src = file_or_stdin(input);
    let dst = file_or_stdout(output);
    tlock_age::decrypt(dst, src, &header.hash(), &beacon.signature()).map(|()| String::from(""))
}
