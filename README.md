# dee: Rust cli for drand

[![Documentation](https://img.shields.io/badge/docs-main-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/dee.svg)
[![crates.io](https://img.shields.io/crates/v/dee.svg)][Crates.io]

[Crates.io]: https://crates.io/crates/dee
[Documentation]: https://docs.rs/dee/

Retrieve public randomness, and encrypt your files to the future. **dee** provides a drand client, and support for timelock encryption.

<p align="center"><img src="./assets/demo.gif?raw=true"/></p>

## Tables of Content

* [Features](#features)
* [What's next](#whats-next)
* [Installation](#installation)
* [Usage](#usage)
  * [Manage remote beacons](#manage-remote-beacons)
  * [Retrieve public randomness](#retrieve-public-randomness)
  * [Timelock encryption](#timelock-encryption)
  * [Common remotes](#common-remotes)
* [Security Considerations](#security-considerations)
* [FAQ](#faq)
* [License](#license)

## Features

* Retrieve drand randomness
* Manages multiple beacons locally
* Timelock encryption and decryption
* Chain and unchained randomness
* Signatures verification on G1 and G2
* Customizable output format
* Cross platform (Linux, Windows, macOS)
* Interroperability with Go and JS implementation
* wasm32 compatible library

## What's next

* P2P randomness retrieval
* Offline timelock decryption

## Installation

| Environment | CLI Command |
|:------------|:------------|
| Cargo (Rust 1.67+) | `cargo install dee --git https://github.com/thibmeu/drand-rs` |

On Linux, Windows, or macOS, you can use the [pre-built binaries](https://github.com/thibmeu/drand-rs/releases).

## Usage

You can use the `--help` option to get more details about the commands and their options.

```bash
dee [OPTIONS] <COMMAND>
```

### Manage remote beacons

Add fastnet remote beacon, and shows details about it.
```bash
dee remote add fastnet https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493
fastnet
```

```bash
dee remote show --long fastnet
URL       : https://drand.cloudflare.com/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493
Public Key: a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e
Period    : 3s
Genesis   : 2023-03-01 15:40:00 UTC
Chain Hash: dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493
Group Hash: a81e9d63f614ccdb144b8ff79fbd4d5a2d22055c0bfe4ee9a8092003dab1c6c0
Scheme ID : bls-unchained-on-g1
Beacon ID : fastnet
```

### Retrieve public randomness

Retrieve round 1000 from fastnet.

```bash
dee rand -u fastnet --long 1000
Round     : 1000
Relative  : 100:09:43 ago
Absolute  : 2023-03-01 16:29:57
Randomness: aa319fc2547e1bdd306633ff63d34e50be76a157477b066906f4d7d63d4e4964
Signature : b09eacd45767c4d58306b98901ad0d6086e2663766f3a4ec71d00cf26f0f49eaf248abc7151c60cf419c4e8b37e80412
```

### Timelock encryption

Encrypt `Hello dee!` string to 30 seconds in the future, using fastnet publickey. If you wait 30 seconds before decrypting, the message is decrypted using the new fastnet signature.

```
echo "Hello dee!" | dee crypt -u fastnet -r 30s > data.dee
dee crypt --decrypt data.dee
Hello dee!
```

### Common remotes

| ID                   | Remote                                                                                          | Timelock encryption |
| :--------------------|:------------------------------------------------------------------------------------------------|:--------------------|
| `fastnet-cloudflare` | `https://drand.cloudflare.com/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493` | Yes                 |
| `fastnet-pl`         | `https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493`         | Yes                 |
| `mainnet-cloudflare` | `https://drand.cloudflare.com`                                                                  | No                  |
| `mainnet-pl`         | `https://api.drand.sh`                                                                          | No                  |

`dee` does not come with a default remote beacon. You should decide whichever suit your needs.

More beacons origin are available on [drand website](https://drand.love/developer/).

## Security Considerations

This software has not been audited. Please use at your sole discretion. With this in mind, dee security relies on the following:
* [tlock: Practical Timelock Encryption from Threshold BLS](https://eprint.iacr.org/2023/189) by Nicolas Gailly, Kelsey Melissaris, and Yolan Romailler, and its implementation in [thibmeu/tlock-rs](https://github.com/thibmeu/tlock-rs),
* [Identity-Based Encryption](https://crypto.stanford.edu/~dabo/papers/bfibe.pdf) by Dan Boneh, and Matthew Franklin, and its implementation in [thibmeu/tlock-rs](https://github.com/thibmeu/tlock-rs),
* The [League of Entropy](https://www.cloudflare.com/leagueofentropy/) to remain honest,
* [age](https://github.com/C2SP/C2SP/blob/main/age.md) encryption protocol, and its implementation in [str4d/rage](https://github.com/str4d/rage),

## FAQ

### Default configuration path

`dee` configuration file is available at the following

| OS      | Path                                                           |
|:--------|:---------------------------------------------------------------|
| Linux   | `/home/alice/.config/dee/default.toml`                         |
| Windows | `C:\Users\Alice\AppData\Roaming\dee\config\default.toml`       |
| macOS   | `/Users/Alice/Library/Application Support/rs.dee/default.toml` |

### Other implementations

drand API specification is at [drand.love/docs/specification](https://drand.love/docs/specification/). drand is based on [Scalable Bias-Resistant Distributed Randomness](https://eprint.iacr.org/2016/1067) by Ewa Syta, Philipp Jovanovic, Eleftherios Kokoris Kogias, Nicolas Gailly, Linus Gasser, Ismail Khoffi, Michael J.  Fischer, and Bryan Ford.
The reference interroperable Go implementation is available at [drand/drand](https://github.com/drand/drand).

timelock encryption was published in [tlock: Practical Timelock Encryption from Threshold BLS](https://eprint.iacr.org/2023/189) by Nicolas Gailly, Kelsey Melissaris, and Yolan Romailler.
The reference interroperable Go implementation is available at [drand/tlock](https://github.com/drand/tlock).

### Rust libraries

dee focuses on building a cli. It relies on Rust libraries to use drand or perform timelock encryption.

If you're looking to implement your own Rust application on top of drand and/or timelock encryption, you can use the following:
* [drand_core](https://github.com/thibmeu/drand-rs/tree/main/drand_core): drand client,
* [tlock](https://github.com/thibmeu/tlock-rs): raw tlock implementation, allowing messages up to 16 bytes,
* [tlock_age](https://github.com/thibmeu/tlock-rs): hybrid encryption, age phassphrase is encrypted using tlock,

## License

This project is under the MIT license.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be MIT licensed as above, without any additional terms or conditions.