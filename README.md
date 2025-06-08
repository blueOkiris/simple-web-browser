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
   - Extensions that can greatly modify the browser (including UI)
2. Decent performance, i.e. built on top 3 engines: Blink, Gecko, or WebKit (WebKit in this case)
3. Well, *except*, not built on Chromium/Blink, since that has a monopoly
4. No added privacy concerns out of the box (no telemetry, ads, or data collection)

Officially supported platform will be Arch Linux

## Build

Dependencies:

- gcc
- gtk3
- make (on Windows, via mingw64)
- pkg-config
- webkit2gtk

\*nix: `make RELEASE=1`

Windows: `mingw32-make RELEASE=1 WINDOWS=1`

## Architecture

Outside of some core framework glue, everything in swb is designed to be a plugin.

Plugins can respond to a variety of events that happen like "when a page loads."

However, plugins can also choose to have a widget, most often a button, in the bar
