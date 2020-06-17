[![Build Status](https://travis-ci.com/avitex/rust-badgen.svg?branch=master)](https://travis-ci.com/avitex/rust-badgen)
[![Crate](https://img.shields.io/crates/v/badgen.svg)](https://crates.io/crates/badgen)
[![Docs](https://docs.rs/badgen/badge.svg)](https://docs.rs/badgen)

# rust-badgen

**Rust SVG badge generator with font path rendering.**  
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

## Benchmarks

Benchmarks were run on an AMD Ryzen 9 3950X on the 17th of Jun 20.

```text
classic                 time:   [4.4839 us 4.4861 us 4.4883 us]                     
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

flat                    time:   [2.9747 us 2.9781 us 2.9813 us]                  
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe

default-slow            time:   [13.276 us 13.286 us 13.297 us]                          
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
```

## Credits

Initially inspired by [github.com/badgen/badgen](https://github.com/badgen/badgen),
but I decided to generate badges in a different manner.
