#!/usr/bin/env sh

# see https://github.com/cross-rs/cross/issues/391
# FOR 32bit cross build --release --target=arm-unknown-linux-gnueabihf
cross build --release --target=aarch64-unknown-linux-gnu

# use ansible
# rsync target/arm-unknown-linux-gnueabihf/release/ip-camera "kpfromer@192.168.0.175:/home/kpfromer/"
