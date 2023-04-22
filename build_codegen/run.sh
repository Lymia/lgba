#!/bin/sh

cd "$(dirname "${BASH_SOURCE[0]}")"

cargo run --target x86_64-unknown-linux-gnu
