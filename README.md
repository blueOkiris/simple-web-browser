# SWB (Simple Web Browser)

## Description


A Web Browser should be a Web Browser and nothing more, and it should integrate well with your system.

This project aims to be a simple GTK Web Browser that will support full website rendering, password/bookmark sync, and ad-block which are viewed as the essential functionality for modern web browsing.

## Build

Install Dependencies:
 - Rustup
 - Gtk3
 - Webkit2Gtk

Do the first time:
`mkdir ~/.swb; cp error-page.html ~/.swb`

`cargo build --release`
