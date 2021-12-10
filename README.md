# SWB (Simple Web Browser)

## Description


A Web Browser should be a Web Browser and nothing more, and it should integrate well with your system.

This project aims to be a simple GTK Web Browser that will support full website rendering, password/bookmark sync, and ad-block which are viewed as the essential functionality for modern web browsing.

TODO:
 - Finish bookmark management and sync
 - Add password syncing

## Build

Install Dependencies:
 - Rustup
 - libgtk3
 - libwebkit2gtk-4.0-dev
 - glib2.0-dev
 - libcairo2-dev
 - libpango1.0-dev
 - libatk1.0-dev
 - libjavascriptcoregtk-dev
 - libgdk-pixbuf2.0-dev
 - libsoup2.4-dev
 - libgdk-3-dev
 - make
 - gcc

Run the build script: `./build.sh`