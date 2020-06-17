#!/bin/sh

cargo run --example classic > ./classic.svg
cargo run --example flat > ./flat.svg
cargo run --example status > ./status.svg