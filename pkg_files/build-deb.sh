#!/bin/bash
# Author: Dylan Turner
# Description:
#  - Helper script to download the git and put it in the needed fldrs
#  - Run from root dir

sudo apt install -y libssl-dev libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.0-dev cargo

mkdir -p swb/opt/swb
mkdir -p swb/opt/swb/plugins
mkdir -p swb/opt/swb/adblock
mkdir -p swb/usr/share/applications
mkdir -p swb/usr/share/pixmaps
mkdir -p swb/usr/local

git clone https://github.com/dudik/blockit
cd blockit
make

cp blockit.so ../swb/opt/swb/adblock
chmod 755 ../swb/opt/swb/adblock/blockit.so

cd ..

git clone https://github.com/blueOkiris/simple-web-browser
cd simple-web-browser
cargo build --release

cp target/release/swb ../swb/opt/swb
cp target/release/libswb_webkit.so ../swb/opt/swb/plugins
cp target/release/libswb_bookmarks.so ../swb/opt/swb/plugins
cp swb-icon.png ../swb/usr/share/pixmaps
cp swb.desktop ../swb/usr/share/applications

cd ..

echo "Installing adblock-rust-server."
cargo install adblock-rust-server
cp ~/.cargo/bin/adblock-rust-server swb/opt/swb/adblock/

cargo uninstall adblock-rust-server
sudo apt remove -y libssl-dev libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.0-dev cargo --auto-remove

dpkg-deb --build swb
