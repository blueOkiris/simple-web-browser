# SWB (Simple Web Browser)

![Screenshot of Simple Web Browser on i3 (i.e. no menu bar)](./screenshot.png)

## Description

Pronounced as "swub" /swʌb/

A Web Browser should be a Web Browser and nothing more, and it should integrate well with your system.

Goals:
1. Built with GTK/Qt to integrate with DEs
2. Adblock and Password/Bookmark sync
3. Mobile client
4. Decent performance (built on top 3 engines: Blink, Gecko, or Webkit)
5. Private out of the box (no telemetry, ads, or data collection)
6. Not built on Chromium

## Build

Obviously, all this will be handled by the package, but if you're building from source, this is what you've gotta do.

First, install Dependencies:
- cargo
- [adblock-rust-server](https://github.com/dudik/blockit)
  + Note #1: Recommended to add "https://easylist-downloads.adblockplus.org/easylist_min_content_blocker.json" to ~/.config/ars/urls
  + Note #2: the directory to place it in is: `adblock/`, so do the `sudo ln -s` to that location
- libgtk3
- Linux

To simply try it out, from the repo folder, run `cargo run`

If you want to install it somewhere:
- Build with `cargo build --release`
- Copy the binary "swb" from target to the install location, /opt/swb/
- Create a plugins/ folder in the install location and copy into it the libswb_bookmarks.so and libswb_webkit.so files from target/release/
- Copy your adblock/ folder containing the blockit.so file into that folder as well
- `sudo ln -s /opt/swb/swb /usr/bin/swb`
- Copy swb.desktop to /usr/share/applications and copy swb-icon.png /usr/share/pixmap

Again, once the package is made, it will do all that jazz for you

## Architecture

The browser itself is built around plugins.

Want to use Firefox sync for bookmarks instead of my system? Go ahead and make a plugin for that

Want a different adblock? Go for it

Want to switch to a completely different browser engine? Be my guest!

The default system will use:
- Custom password and bookmark sync
- Blockit adblocker combined into a plugin with Webkit Gtk

The actual main window contains:
- a container for the web page rendering
- back and forward buttons
- a search bar
- a refresh button
- a bookmarks menu including a folders system
- a sync menu for loggin in/logging out

Events from the main window like searching or creating a bookmark call plugins to do the dirty work. To see what the plugin supports, look in `core/src/plugin.rs`

The mobile app has yet to be thought about. Might go the easy route and just try to compile for WebAsm, create a webview wrapper for android, and get it to run in that, or I might try to actually make a new interface. Has not been decided yet.
