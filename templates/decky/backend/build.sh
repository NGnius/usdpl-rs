#!/bin/bash

cargo build --release --features ,$1
mkdir -p ../bin
# TODO replace "backend" \/ with binary name
cp ./target/release/backend ../bin/backend
