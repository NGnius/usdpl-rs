#!/bin/bash
if [ -n "$1" ]; then
    if [ "$1" == "--help" ]; then
        echo "Usage:
$0 [decky|crankshaft|<nothing>]"
        exit 0
    elif [ "$1" == "decky" ]; then
        echo "Building back-end module for decky framework"
        cargo build --release --features decky
    elif [ "$1" == "crankshaft" ]; then
        echo "WARNING: crankshaft support is unimplemented"
        cargo build --release --features crankshaft
    else
        echo "Unsupported plugin framework \`$1\`"
        exit 1
    fi
else
    echo "WARNING: Building for any plugin framework, which may not work for every framework"
    cargo build --release
fi
