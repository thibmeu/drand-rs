---
marp: true
paginate: true
_paginate: false
---

# Building `dee`, a simple timelock client

Thibault Meunier

---

## In the next 15min

1. Demo
2. CLI design
3. Timelock API
4. Final words

---

# Demo

_Try it at home_

---

## Live demo

Installation

```bash
cargo install dee
```

Add a remote chain

```bash
dee remote add fastnet https://drand.cloudflare.com/dbd506d...
fastnet
```

---

## Live demo - 2

Retrieve public randomness

```bash
dee rand -u fastnet
3129db460507ff559f7fa5e71d6f8bc66aec27516de3d78f7461f6299a2bd483
```

Encrypt 30 seconds to the future

```
echo "Hello dee!" | dee crypt -r 30s > locked.dee
```

Decrypt, the future is now

```
dee crypt --decrypt locked.dee
Hello dee!
```

---

# Designing a CLI

_CLI experience is real_

---
<!-- header: Designing a CLI-->

## Limit default

No default network

```bash
dee remote add mainnet https://api.drand.sh
```

Choose your own
```bash
dee rand --set-upstream mainnet
```

---

## Communication for everyone

Configurable output level

```bash
dee rand -l
Round     : 2820083
Relative  : 00:00:24 ago
Absolute  : 2023-03-28 19:58:30
Randomness: 66aba01bb54f200ef6363143615e1e193eaacbb89dcc7b38...
Signature : 82fb1e24bd603216241d75d51c3378b193d62e4fb8fdbeab...
```

Informative error

```bash
echo "Hello world!" | dee crypt -r 30s            
error: remote must use unchained signatures
```

---

## Mimic existing CLIs

git inspired

```bash
dee remote show mainnet
```

age inspired

```bash
dee crypt --decrypt --armor < cat.png
```

drand inspired

```bash
dee rand -u mainnet --json 1000
```

---

## Rust specific devtooling

[clap](https://docs.rs/clap/latest/clap/) all in one argument parser, documentation, and manpages generation
```rust
/// Set default upstream. If empty, use the latest upstream.
#[arg(short = 'u', long, value_hint = ValueHint::Url)]
set_upstream: Option<String>,
```

Cross-platform support is simpler without openssl

```rust
cargo build --target wasm32-wasi
```

Considered two BLS12-381 libraries: [zkcrypto/bls12_381](https://github.com/zkcrypto/bls12_381) and [arkworks-rs/curves](https://github.com/arkworks-rs/curves).

```bash
cargo bench --all-features
```

---

<!-- header: '' -->

# Timelock API

_Encrypting towards the future doesn't negate API considerations_

---

<!-- header: Timelock API -->

## Work offline

Go

```go
func (t Tlock) Encrypt(
  dst io.Writer, src io.Reader, roundNumber uint64
) (err error) {
```

Rust

```rust
fn encrypt(
  dst: Write, mut src: Read, roundNumber: u64,
  hash: &[u8], pk: &[u8],
) -> Result<()> {
```

---

## Work offline

Go

```go
network := "https://api.drand.sh"
tlock := tlock.New(network)
tlock.Encrypt(dst, src, roundNumber)
```

Rust
```rust
let chain = Chain::new("https://api.drand.sh");
let client = HttpClient::new(chain, None);
let info = client.chain().info().await?;

tlock_age::encrypt(
    &mut dst,
    src,
    &info.hash(),
    &info.public_key(),
    roundNumber,
)?;
```

---

## Interroperability

Two existing implementations: [drand/tlock](drand/tlock) (Go), [drand/tlock-js](https://github.com/drand/tlock-js) (JavaScript).

[rage](https://github.com/str4d/rage) (Rust implementation of age) adds a [grease stanza](https://github.com/str4d/rage/pull/365): `<rand>-grease <rand>`.

[Hash to curve RFC](https://datatracker.ietf.org/doc/draft-irtf-cfrg-hash-to-curve/) is a beacon of light: [hash_to_field](https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-hash_to_field-implementatio),  [expand_message](https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-expand_message).

Elliptic curve serialisation is not standardised.
$$
\begin{align}
  \mathbb{F}_{p^{12}} \rightarrow c_0 \| c_1 &&
  \mathbb{F}_{p^{12}} \rightarrow c_1 \| c_0
\end{align}
$$
$$
\begin{align}
  c_0 \rightarrow \text{big-endian} &&
  c_0 \rightarrow \text{little-endian}
\end{align}
$$


---

<!-- header: '' -->

# Final words

_Time to move on_

---

<!-- header: Final words -->

## What could be different

Hostname instead of chain hash

```bash
https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc...
-> https://fastnet.api.drand.sh
```

Stanza format

```
tlock {round} {chain_hash}
-> tlock REDACTED REDACTED
```

Stateless CLI

```
dee remote
-> dee rand -u https://api.drand.sh/<hash>
-> DEE_REMOTE=https://api.drand.sh/<hash>
```

---

## Takeaways

1. A new [drand](https://github.com/thibmeu/drand-rs) and [tlock](https://github.com/thibmeu/tlock-rs) implementation.
2. One [academic paper](https://eprint.iacr.org/2023/189), multiple engineering tradeoffs.
3. tlock is not be constrained to existing drand API.
4. [Discussions](https://join.slack.com/t/drandworkspace/shared_invite/zt-19u4rf6if-bf7lxIvF2zYn4~TrBwfkiA) improve software. Thanks to everyone that answered questions.

---

<!-- header: '' -->

# Thank you

For more information, go to:
[github.com/thibmeu/drand-rs](https://github.com/thibmeu/drand-rs)
[github.com/thibmeu/tlock-rs](https://github.com/thibmeu/tlock-rs)