# SWB (Simple Web Browser)

## Description

Pronounced as "swub" /swʌb/

A Web Browser should be a Web Browser and nothing more, and it should integrate well with your system.

Goals:
1. Only the folowing features implemented (what I consider necessary):
   - Modern rendering of pages (Doesn't have to be perfect, but must be usable)
   - Navigation: Search, Direct Link, Forwards, Backwards
   - Tabs
   - Downloads
   - Adblock
   - Bookmarks: Creation, Deletion, Folder Sorting
   - Password Storage
2. Plugin system that can greatly modify the browser (including UI), acheived by making *everything* a plugin, even things like the back and forward buttons.
3. Decent performance, i.e. built on top 3 engines: Blink, Gecko, or WebKit (WebKit in this case)
4. Well, *except*, not built on Chromium/Blink, since that has a monopoly
5. No added privacy concerns out of the box (no telemetry, ads, or data collection)

Officially supported platform will be Arch Linux, but binaries will be built for other Linux, and package maintainers are welcome to build their own.

## Build

Dependencies:

- curl
- \*nix (Arch is the supported platform)
- gcc
- gtk3
- make
- pkg-config
- webkit2gtk

`make RELEASE=1`

## Architecture

Outside of some core framework glue, everything in swb is designed to be a plugin.

Plugins can respond to a variety of events that happen like "when a page loads."

However, plugins can also choose to have a widget, most often a button, in the bar
