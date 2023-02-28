use anyhow::{anyhow, Result};
/// Verify BLS Signatures used in drand
/// inspired from https://github.com/noislabs/drand-verify/blob/1017235f6bcfcc9fb433926c0dc1b9a013bd4df3/src/verify.rs#L58
use bls12_381::{
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    Bls12, G1Affine, G1Projective, G2Affine, G2Prepared, G2Projective,
};
use pairing::{group::Group, MultiMillerLoop};

const DOMAIN: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

/// Check that signature is the actual aggregate of message and public key.
/// Calculated by `e(g2, signature) == e(pk, hash)`.
/// `signature` and `hash` are on G2, `public_key` is on G1.
pub fn verify(signature: &[u8], hash: &[u8], public_key: &[u8]) -> Result<bool> {
    // 48 is bytes of G1
    // G1Affine::identity().to_compressed().len()
    if signature.len() == 48 {
        verify_g1_on_g2(signature, hash, public_key)
    } else {
        verify_g2_on_g1(signature, hash, public_key)
    }
}

/// Check that signature is the actual aggregate of message and public key.
/// Calculated by `e(g2, signature) == e(pk, hash)`.
/// `signature` and `hash` are on G2, `public_key` is on G1.
pub fn verify_g2_on_g1(signature: &[u8], hash: &[u8], public_key: &[u8]) -> Result<bool> {
    let g: G2Projective = HashToCurve::<ExpandMsgXmd<sha2::Sha256>>::hash_to_curve(hash, DOMAIN);
    let hash_on_curve = G2Affine::from(g);
    let g1 = G1Affine::generator();
    let sigma = match g2_from_variable(signature) {
        Ok(sigma) => sigma,
        Err(err) => return Err(anyhow!("Verification Error: {}", err)),
    };
    let r = match g1_from_variable(public_key) {
        Ok(r) => r,
        Err(err) => return Err(anyhow!("Verification Error: {}", err)),
    };
    Ok(fast_pairing_equality(&g1, &sigma, &r, &hash_on_curve))
}

/// Check that signature is the actual aggregate of message and public key.
/// Calculated by `e(g1, signature) == e(pk, hash)`.
/// `signature` is on G1, `public_key` and `hash` are on G2.
pub fn verify_g1_on_g2(signature: &[u8], hash: &[u8], public_key: &[u8]) -> Result<bool> {
    let g: G1Projective = HashToCurve::<ExpandMsgXmd<sha2::Sha256>>::hash_to_curve(hash, DOMAIN);
    let hash_on_curve = G1Affine::from(g);
    let g2 = G2Affine::generator();
    let sigma = match g1_from_variable(signature) {
        Ok(sigma) => sigma,
        Err(err) => return Err(anyhow!("Verification Error: {}", err)),
    };
    let s = match g2_from_variable(public_key) {
        Ok(r) => r,
        Err(err) => return Err(anyhow!("Verification Error: {}", err)),
    };
    Ok(fast_pairing_equality(&sigma, &g2, &hash_on_curve, &s))
}

/// Checks if e(p, q) == e(r, s)
///
/// See https://hackmd.io/@benjaminion/bls12-381#Final-exponentiation.
///
/// Optimized by this trick:
///   Instead of doing e(a,b) (in G2) multiplied by e(-c,d) (in G2)
///   (which is costly is to multiply in G2 because these are very big numbers)
///   we can do FinalExponentiation(MillerLoop( [a,b], [-c,d] )) which is the same
///   in an optimized way.
fn fast_pairing_equality(p: &G1Affine, q: &G2Affine, r: &G1Affine, s: &G2Affine) -> bool {
    let minus_p = -p;
    // "some number of (G1, G2) pairs" are the inputs of the miller loop
    let pair1 = (&minus_p, &G2Prepared::from(*q));
    let pair2 = (r, &G2Prepared::from(*s));
    let looped = Bls12::multi_miller_loop(&[pair1, pair2]);
    // let looped = Bls12::miller_loop([&pair1, &pair2]);
    let value = looped.final_exponentiation();
    value.is_identity().into()
}

fn g1_from_variable(data: &[u8]) -> Result<G1Affine> {
    if data.len() != 48 {
        return Err(anyhow!("Invalid Point"));
    }

    let mut buf = [0u8; 48];
    buf[..].clone_from_slice(data);
    g1_from_fixed(buf)
}

fn g2_from_variable(data: &[u8]) -> Result<G2Affine> {
    if data.len() != 96 {
        return Err(anyhow!("Invalid Point"));
    }

    let mut buf = [0u8; 96];
    buf[..].clone_from_slice(data);
    g2_from_fixed(buf)
}

fn g1_from_fixed(data: [u8; 48]) -> Result<G1Affine> {
    Option::from(G1Affine::from_compressed(&data)).ok_or(anyhow!("Decoding Error"))
}

fn g2_from_fixed(data: [u8; 96]) -> Result<G2Affine> {
    Option::from(G2Affine::from_compressed(&data)).ok_or(anyhow!("Decoding Error"))
}
