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

| Environment        | CLI Command                                                   |
|:-------------------|:--------------------------------------------------------------|
| Cargo (Rust 1.74+) | `cargo install dee --git https://github.com/thibmeu/drand-rs` |

On Linux, Windows, or macOS, you can use the [pre-built binaries](https://github.com/thibmeu/drand-rs/releases).

## Usage

You can use the `--help` option to get more details about the commands and their options.

```bash
dee [OPTIONS] <COMMAND>
```

### Manage remote beacons

Add quicknet remote beacon, and shows details about it.
```bash
dee remote add quicknet https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971
quicknet
```

```bash
dee remote show --long quicknet
URL       : https://drand.cloudflare.com/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971
Public Key: 83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a
Period    : 3s
Genesis   : 2023-08-23 15:09:27.0 +00:00:00
Chain Hash: 52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971
Group Hash: f477d5c89f21a17c863a7f937c6a6d15859414d2be09cd448d4279af331c5d3e
Scheme ID : bls-unchained-g1-rfc9380
Beacon ID : quicknet
```

### Retrieve public randomness

Retrieve round 1000 from quicknet.

```bash
dee rand -u quicknet --long 1000
Round     : 1000
Relative  : 100:09:43 ago
Absolute  : 2023-08-23 15:59:24
Randomness: fe290beca10872ef2fb164d2aa4442de4566183ec51c56ff3cd603d930e54fdd
Signature : b44679b9a59af2ec876b1a6b1ad52ea9b1615fc3982b19576350f93447cb1125e342b73a8dd2bacbe47e4b6b63ed5e39
```

### Timelock encryption

Encrypt `Hello dee!` string to 30 seconds in the future, using quicknet publickey. If you wait 30 seconds before decrypting, the message is decrypted using the new quicknet signature.

```
echo 'Hello dee!' | dee crypt -u quicknet -r 30s > data.dee
dee crypt --decrypt data.dee
Hello dee!
```

### Common remotes

| ID                    | Remote                                                                                          | Timelock encryption |
| :---------------------|:------------------------------------------------------------------------------------------------|:--------------------|
| `quicknet-cloudflare` | `https://drand.cloudflare.com/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971` | Yes                 |
| `quicknet-pl`         | `https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971`         | Yes                 |
| `mainnet-cloudflare`  | `https://drand.cloudflare.com`                                                                  | No                  |
| `mainnet-pl`          | `https://api.drand.sh`                                                                          | No                  |

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
