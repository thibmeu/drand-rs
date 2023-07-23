# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
