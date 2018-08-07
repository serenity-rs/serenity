#!/usr/bin/env bash

if [ ! -d "$HOME/libsodium/lib" ]; then
    wget https://github.com/jedisct1/libsodium/releases/download/1.0.16/libsodium-1.0.16.tar.gz
    tar zxvf libsodium-1.0.16.tar.gz
    cd libsodium-1.0.16
    ./configure --prefix="$HOME/libsodium"
    make
    make install
else
    echo 'Using cached libsodium directory'
fi
