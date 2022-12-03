#!/bin/bash

cargo build --target x86_64-unknown-linux-musl --features encrypt
mkdir -p ../bin
# TODO replace "backend" \/ with binary name
cp ./target/release/backend ../bin/backend
