# Changelog

All notable changes to this project will be documented in this file. Changes to the [drand_core crate](../drand_core/CHANGELOG.md) also apply to the dee CLI tools, and are not duplicated here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.8] - 2023-07-23

### Fix

- Fix typo in `dee rand` help message

## [0.0.7] - 2023-04-10

### Added

- Add documentation for design considerations

### Changed

- Update dependencies
- Update time library from chrono to time
- Update dee error handling on HTTP remotes
- Update tlock_age to v0.0.2 for improved performance

### Fix

- Fix Chain error handling on wrong URL string

## [0.0.6] - 2023-03-22

### Added

- Decryption from stdin
- Compatibility with [drand/tlock](https://github.com/drand/tlock) and [drand/tlock-js](https://github.com/drand/tlock)

### Changed
- README to detail current and planned features, as well as references
