use anyhow::Result;
use serde::{Serialize as SerdeSerialize, Deserialize as SerdeDeserialize};
use sha2::{Sha256, Digest};

use crate::chain::ChainInfo;


#[derive(Debug, SerdeSerialize, SerdeDeserialize)]
#[serde(untagged)]
pub enum RandomnessBeacon {
    ChainedBeacon(ChainedBeacon),
    UnchainedBeacon(UnchainedBeacon)
}

impl RandomnessBeacon {
    pub fn verify(&self, info: ChainInfo) -> Result<bool> {
      if info.scheme_id() != "pedersen-bls-chained" {
        return Ok(false);
      }

      let message = self.message()?;
      let public_key = hex::decode(info.public_key())?;
      let signature = hex::decode(self.signature())?;
      let signature_verify = crate::verify::verify(&signature, &message, &public_key)?;

      let mut hasher = Sha256::new();
      hasher.update(hex::decode(self.signature())?);
      let expected_randomness = hasher.finalize().to_vec();
      let epoch_randomness = hex::decode(self.randomness())?;
      let randomness_verify = expected_randomness == epoch_randomness;

      Ok(signature_verify && randomness_verify)
    }

    pub fn round(&self) -> u64 {
      match self {
        Self::ChainedBeacon(chained) => chained.round,
        Self::UnchainedBeacon(unchained) => unchained.round,
      }
    }

    pub fn randomness(&self) -> String {
      match self {
        Self::ChainedBeacon(chained) => chained.randomness.clone(),
        Self::UnchainedBeacon(unchained) => unchained.randomness.clone(),
      }
    }

    pub fn signature(&self) -> String {
      match self {
        Self::ChainedBeacon(chained) => chained.signature.clone(),
        Self::UnchainedBeacon(unchained) => unchained.signature.clone(),
      }
    }
}

impl Message for RandomnessBeacon {
  fn message(&self) -> Result<Vec<u8>> {
    match self {
      Self::ChainedBeacon(chained) => chained.message(),
      Self::UnchainedBeacon(unchained) => unchained.message(),
    }
  }
}

trait Message {
  fn message(&self) -> Result<Vec<u8>>;
}

#[derive(Debug, SerdeSerialize, SerdeDeserialize)]
pub struct ChainedBeacon {
    round: u64,
    randomness: String,
    signature: String,
    previous_signature: String
}

impl Message for ChainedBeacon {
  fn message(&self) -> Result<Vec<u8>> {
    let mut buf = [0; 96+8];
    let (signature_buf, round_buf) = buf.split_at_mut(96);

    hex::decode_to_slice(self.previous_signature.as_str(), signature_buf)?;
    round_buf.clone_from_slice(self.round.to_be_bytes().as_ref());

    let mut hasher = Sha256::new();
    hasher.update(buf);
    Ok(hasher.finalize().to_vec())
  }
}

#[derive(Debug, SerdeSerialize, SerdeDeserialize)]
pub struct UnchainedBeacon {
    round: u64,
    randomness: String,
    signature: String
}

impl Message for UnchainedBeacon {
  fn message(&self) -> Result<Vec<u8>> {
    let buf = self.round.to_be_bytes();

    let mut hasher = Sha256::new();
    hasher.update(buf);
    Ok(hasher.finalize().to_vec())
  }
}