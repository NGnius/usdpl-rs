#!/bin/bash

echo "--- Building a new encrypted USDPL plugin for Decky loader ---"
echo "This script assumes you have a functioning cargo (Rust) and pnpm (Node/Javascript) setup"
echo "If you do not, parts of this script will not work correctly (but may still exit 0)"

export USDPL_ENCRYPTION_KEY=$(openssl enc -aes-256-cbc -k caylon -pbkdf2 -P -md sha1 | awk -F= '{if ($1 == "key") print $2}')
echo "Key generated..."
#echo USDPL key: $USDPL_ENCRYPTION_KEY

echo "Building backend..."
cd ./backend && ./build.sh decky,encrypt && cd ..

echo "Rebuilding USDPL frontend..."
cd ./src/usdpl-front && ./rebuild.sh decky encrypt && cd ../..

echo "Building frontend..."
# pnpm does not like local dependencies, and doesn't install them unless forced to install everything
rm -rf ./node_modules && pnpm install && pnpm run build
