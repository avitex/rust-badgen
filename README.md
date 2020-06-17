[![Build Status](https://travis-ci.com/avitex/rust-badgen.svg?branch=master)](https://travis-ci.com/avitex/rust-badgen)
[![Crate](https://img.shields.io/crates/v/badgen.svg)](https://crates.io/crates/badgen)
[![Docs](https://docs.rs/badgen/badge.svg)](https://docs.rs/badgen)

# rust-badgen

**A Rust implementation of [github.com/badgen/badgen](https://github.com/badgen/badgen).**  
Documentation hosted on [docs.rs](https://docs.rs/badgen).

```toml
[dependencies]
badgen = "0.1"
```

## Examples

**Classic**

![Classic](./data/generated/classic.svg)

```rust
let badge = badgen::badge(&badgen::Style::classic(), "4.2 KB", Some("minzipped size")).unwrap();
println!("{}", badge);
```

**Flat**

![Classic](./data/generated/flat.svg)

```rust
let badge = badgen::badge(&badgen::Style::flat(), "4.2 KB", Some("minzipped size")).unwrap();
println!("{}", badge);
```
