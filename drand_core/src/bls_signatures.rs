/// Verify BLS Signatures used in drand
/// inspired from https://github.com/noislabs/drand-verify/blob/1017235f6bcfcc9fb433926c0dc1b9a013bd4df3/src/verify.rs#L58
use std::ops::Neg;

use anyhow::{anyhow, Result};
use ark_bls12_381::{g1, g2, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{
    bls12::Bls12,
    hashing::{curve_maps::wb::WBMap, map_to_curve_hasher::MapToCurveBasedHasher, HashToCurve},
    models::short_weierstrass,
    pairing::Pairing,
    AffineRepr, CurveGroup,
};
use ark_ff::{field_hashers::DefaultFieldHasher, Zero};
use ark_serialize::CanonicalDeserialize;

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
    let mapper = MapToCurveBasedHasher::<
        short_weierstrass::Projective<g2::Config>,
        DefaultFieldHasher<sha2::Sha256, 128>,
        WBMap<g2::Config>,
    >::new(DOMAIN)
    .map_err(|_| anyhow!("cannot initialise mapper for sha2 to BLS12-381 G1"))?;
    let hash_on_curve = G2Projective::from(
        mapper
            .hash(hash)
            .map_err(|_| anyhow!("hash cannot be mapped to G1"))?,
    )
    .into_affine();

    let g1 = G1Affine::generator();
    let sigma = g2_from_variable(signature).map_err(|e| anyhow!("verification Error: {}", e))?;
    let r = g1_from_variable(public_key).map_err(|e| anyhow!("verification Error: {}", e))?;
    Ok(fast_pairing_equality(&g1, &sigma, &r, &hash_on_curve))
}

/// Check that signature is the actual aggregate of message and public key.
/// Calculated by `e(g1, signature) == e(pk, hash)`.
/// `signature` is on G1, `public_key` and `hash` are on G2.
pub fn verify_g1_on_g2(signature: &[u8], hash: &[u8], public_key: &[u8]) -> Result<bool> {
    let mapper = MapToCurveBasedHasher::<
        short_weierstrass::Projective<g1::Config>,
        DefaultFieldHasher<sha2::Sha256, 128>,
        WBMap<g1::Config>,
    >::new(DOMAIN)
    .map_err(|_| anyhow!("cannot initialise mapper for sha2 to BLS12-381 G1"))?;
    let hash_on_curve = G1Projective::from(
        mapper
            .hash(hash)
            .map_err(|_| anyhow!("hash cannot be mapped to G1"))?,
    )
    .into_affine();

    let g2 = G2Affine::generator();
    let sigma = g1_from_variable(signature).map_err(|e| anyhow!("verification Error: {}", e))?;
    let s = g2_from_variable(public_key).map_err(|e| anyhow!("verification Error: {}", e))?;
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
    let minus_p = p.neg();
    // "some number of (G1, G2) pairs" are the inputs of the miller loop
    let looped = Bls12::<ark_bls12_381::Config>::multi_miller_loop([minus_p, *r], [*q, *s]);
    let value = Bls12::final_exponentiation(looped);
    value.unwrap().is_zero()
}

fn g1_from_variable(data: &[u8]) -> Result<G1Affine> {
    if data.len() != 48 {
        return Err(anyhow!("Invalid Point"));
    }

    G1Affine::deserialize_compressed(data).map_err(|_| anyhow!("deserialization failed"))
}

fn g2_from_variable(data: &[u8]) -> Result<G2Affine> {
    if data.len() != 96 {
        return Err(anyhow!("Invalid Point"));
    }

    G2Affine::deserialize_compressed(data).map_err(|_| anyhow!("deserialization failed"))
}
