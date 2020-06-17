[![Build Status](https://travis-ci.com/avitex/rust-badgen.svg?branch=master)](https://travis-ci.com/avitex/rust-badgen)
[![Crate](https://img.shields.io/crates/v/badgen.svg)](https://crates.io/crates/badgen)
[![Docs](https://docs.rs/badgen/badge.svg)](https://docs.rs/badgen)
![Status](./data/generated/status.svg)

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

![Flat](./data/generated/flat.svg)

```rust
let badge = badgen::badge(&badgen::Style::flat(), "4.2 KB", Some("minzipped size")).unwrap();
println!("{}", badge);
```

## Benchmarks

Benchmarks were run on an AMD Ryzen 9 3950X on the 17th of Jun 20.

```text
classic                 time:   [1.8844 us 1.8888 us 1.8933 us]                     
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild

flat                    time:   [1.3516 us 1.3536 us 1.3556 us]                  
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild

default-slow            time:   [11.833 us 11.847 us 11.863 us]                          
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
```

## Credits

Initially inspired by [github.com/badgen/badgen](https://github.com/badgen/badgen),
but I decided to generate badges in a different manner.
