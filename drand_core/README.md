# drand-core: Rust implementation of drand

[![Documentation](https://img.shields.io/badge/docs-main-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/drand_core.svg)
[![crates.io](https://img.shields.io/crates/v/drand_core.svg)][Crates.io]

[Crates.io]: https://crates.io/crates/drand_core
[Documentation]: https://docs.rs/drand_core/

drand-rs is a tool to retrieve public randomness generated by drand beacon. It features an HTTP client, and verification method.

The format specification is at [drand.love/docs/specification](https://drand.love/docs/specification/). drand was designed in [Scalable Bias-Resistant Distributed Randomness](https://eprint.iacr.org/2016/1067.pdf).

The reference interroperable Go implementation is available at [drand/drand](https://github.com/drand/drand).

## Usage

```rust
use drand_core::{chain, http_chain_client};

let chain = chain::Chain::new("https://drand.cloudflare.com");

let client = http_chain_client::HttpChainClient::new(chain, None);

let latest = client.latest().await?;
```