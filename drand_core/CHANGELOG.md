# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add a check on the returned round value

### Changed

- Update dependencies
- Update Rust to 1.74

## [0.0.14] - 2023-08-30

### Added

- Dedicated error type
- Add use of OS certificate store when available

## [0.0.13] - 2023-08-23

### Added

- Add rfc9380 helper method for ChainInfo

### Fix

- Scheme ID of RFC 9380 is bls-unchained-g1-rfc9380

## [0.0.11] - 2023-08-08

### Changed

- Public struct for chain time info

## [0.0.10] - 2023-08-08

### Added

- Built-in beacon time estimation

### Fix

- Wasm32 build

## [0.0.9] - 2023-08-01

### Changed

- Update HTTP Client to use ureq

### Fix

- Fix unecessary ark_bls12_381 boilerplate
- Fix G1 -> G2 in error messages

## [0.0.8] - 2023-07-23

### Added

- Add G1 scheme conformant to Hash to Curve RFC

## [0.0.7] - 2023-04-10

### Added

- Add coin_clip example for drand_core
- Add `get_by_unix_time` method on drand HttpClient

### Changed

- Update dependencies
- Update drand_core public API to expose HttpClient
- Update bls12-381 library to arkworks/curves

### Fix

- Fix Chain error handling on wrong URL string

### Remove

- Remove Chain struct abstraction

## [0.0.6] - 2023-03-22

### Changed

- README to detail current and planned features
