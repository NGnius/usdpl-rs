#!/bin/bash

export USDPL_ENCRYPTION_KEY=$(openssl enc -aes-256-cbc -k caylon -pbkdf2 -P -md sha1 | awk -F= '{if ($1 == "key") print $2}')
echo USDPL key: $USDPL_ENCRYPTION_KEY

cd ./backend && ./build.sh && cd ..

cd ./src/usdpl_front && ./rebuild.sh decky encrypt && cd ../..
npm install
npm run build
