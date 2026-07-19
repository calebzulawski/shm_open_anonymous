shm_open_anonymous
==================
[![CI](https://github.com/calebzulawski/shm_open_anonymous/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/calebzulawski/shm_open_anonymous/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/shm_open_anonymous)](https://crates.io/crates/shm_open_anonymous)
[![Crates.io](https://img.shields.io/crates/v/shm_open_anonymous)](https://crates.io/crates/shm_open_anonymous)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/shm_open_anonymous)

Create anonymous POSIX shared memory objects.

This crate only works on `unix` targets and is `no_std` compatible.

The minimum supported Rust version tracks stable Rust minus two release trains
(currently Rust 1.95).

Inspired by the C library [`shm_open_anon`](https://github.com/lassik/shm_open_anon).

## License
shm_open_anonymous is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
