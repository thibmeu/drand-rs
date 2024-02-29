use drand_core::HttpClient;
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Flip a coin using the latest drand beacon.
/// The output is deterministic, and based on this latest beacon.
fn main() {
    // Create a new client and retrieve the latest beacon. By default, it verifies its signature against the chain info.
    let client: HttpClient = "https://api.drand.sh".try_into().unwrap();
    let latest = client.latest().unwrap();
    let round = latest.round();

    // Create a new seeded RNG. For a given beacon, the coin flip result is deterministic.
    let seed: <ChaCha20Rng as SeedableRng>::Seed = latest.randomness().try_into().unwrap();
    let mut rng = ChaCha20Rng::from_seed(seed);

    // Flip a coin using the seeded RNG.
    let coin = ["HEAD", "TAIL"];
    let flip = coin.choose(&mut rng).unwrap();
    println!("{flip} (round {round})");
}
