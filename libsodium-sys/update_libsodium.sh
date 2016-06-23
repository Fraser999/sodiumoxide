#!/bin/bash

# Running this script will replace the contents of the `libsodium` folder and update `sources.txt`
# (which is used in `build.rs`).  To change to a new version of libsodium, it should only be
# necessary to set the `Version` variable below and run this script.

Version=1.0.10

# Stop the script if any command fails
set -o errtrace
trap 'exit' ERR

Root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

mkdir -p "$Root/temp"
cd "$Root/temp"
wget https://github.com/jedisct1/libsodium/releases/download/$Version/libsodium-$Version.tar.gz
wget https://github.com/jedisct1/libsodium/releases/download/$Version/libsodium-$Version.tar.gz.minisig
../minisign -Vm libsodium-$Version.tar.gz -P RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3
tar xfz libsodium-$Version.tar.gz

mkdir -p libsodium
cd libsodium-$Version
cp --parents `find src -name *.c -o -name *.h` ../libsodium
cp AUTHORS ChangeLog LICENSE README README.markdown THANKS ../libsodium
./configure  # to generate version.h
cp --parents src/libsodium/include/sodium/version.h ../libsodium

cd "$Root"
rm -rf libsodium
mv temp/libsodium .
rm -rf temp
find libsodium -name *.c > sources.txt
