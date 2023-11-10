#!/bin/bash

# use libafb development version if any
export LD_LIBRARY_PATH="/usr/local/lib64:$LD_LIBRARY_PATH"
export PATH="/usr/local/lib64:$PATH"
clear

if ! test -f $CARGO_TARGET_DIR/debug/libti_am62x.so; then
    echo "FATAL: missing libafb_nfc.so use: cargo build"
    exit 1
fi

# start binder with test config
afb-binder -vvv --binding=$CARGO_TARGET_DIR/debug/libti_am62x.so
