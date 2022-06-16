#!/bin/bash
if [ -n "$1" ]; then
    if [ "$1" == "--help" ]; then
        echo "Usage:
$0 [decky|crankshaft|<nothing>]"
        exit 0
    elif [ "$1" == "decky" ]; then
        echo "Building back & front for decky framework"
        # usdpl-back
        echo "...Running usdpl-back build..."
        cd ./usdpl-back
        ./build.sh decky
        # usdpl-front
        echo "...Running usdpl-front build..."
        cd ../usdpl-front
        ./build.sh decky
        cd ..
        echo "Built usdpl back & front for decky"
    elif [ "$1" == "crankshaft" ]; then
        echo "WARNING: crankshaft is unimplemented"
        echo "Building back & front for crankshaft framework"
        # usdpl-back
        cd ./usdpl-back
        ./build.sh crankshaft
        # usdpl-front
        cd ../usdpl-front
        ./build.sh crankshaft
        cd ..
        echo "Built usdpl back & front for crankshaft"
    else
        echo "Unsupported plugin framework \`$1\`"
        exit 1
    fi
else
    echo "WARNING: Building for any plugin framework, which may not work for every framework"
    echo "Building back & front for any framework"
    # usdpl-back
    echo "...Running usdpl-back build..."
    cd ./usdpl-back
    ./build.sh
    # usdpl-front
    echo "...Running usdpl-front build..."
    cd ../usdpl-front
    ./build.sh
    cd ..
    echo "Built usdpl back & front for any"
fi
