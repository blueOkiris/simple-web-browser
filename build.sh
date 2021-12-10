#!/bin/bash

# Author: Dylan Turner
# Description: Build all sub-projects for the simple web browser

function yes_or_no {
    while true; do
        read -p "$* Is this okay? [y/n]: " yn
        case $yn in
            [Yy]*) return 0;;
            [Nn]*) echo "Aborted"; return 1;;
        esac
    done
}

echo "Initializing adblocker 'blockit' submodule."
git submodule init --
echo "Done."

echo "Installing adblock server."
cargo install adblock-rust-server
echo "Done."

yes_or_no "Next command, installing adblocker, requires sudo."
echo "Installing adblock."
sudo make -C blockit install
echo "Done."

echo "Creating extension directory."
mkdir -p ~/.swb-extensions
echo "Done."

yes_or_no "Next command, link adblocker to extensions directory, requires sudo."
echo "Linking adblock."
sudo ln -s /usr/local/lib/blockit.so ~/.swb-extensions
echo "Done."

echo "Building browser."
cargo build --release
cp target/release/swb .
chmod +x swb
echo "Done."
