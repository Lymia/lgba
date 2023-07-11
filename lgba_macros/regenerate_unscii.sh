#!/bin/sh

cd "$(dirname "${BASH_SOURCE[0]}")"
cargo run --example translate_misaki_bdf
rm -rf target
