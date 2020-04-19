[![Build Status](https://travis-ci.com/avitex/rust-badgen.svg?branch=master)](https://travis-ci.com/avitex/rust-badgen)
[![Crate](https://img.shields.io/crates/v/badgen.svg)](https://crates.io/crates/badgen)
[![Docs](https://docs.rs/badgen/badge.svg)](https://docs.rs/badgen)

# rust-badgen

**A Rust implementation of [github.com/badgen/badgen](https://github.com/badgen/badgen).**  
Documentation hosted on [docs.rs](https://docs.rs/arae).

```toml
[dependencies]
badgen = "0.1.0"
```

## Example

```rust
// Classic style
let badge = badgen::Builder::new("downloads", "12").build().unwrap();
println!("{}", badge);

// Flat style
let badge = badgen::Builder::new("downloads", "12").flat().build().unwrap();
println!("{}", badge);
```
