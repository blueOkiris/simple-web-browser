{ pkgs ? import <nixpkgs> { } }:

let
    unstable = import
        (fetchTarball "https://nixos.org/channels/nixos-unstable/nixexprs.tar.xz") {};
    buildLibs = with pkgs; (with xorg; [
        gtk2
        gtk3
        gdk-pixbuf
        libsoup
        xorg.libX11
        xorg.libXcursor
        libxkbcommon
        xorg.libXrandr
        xorg.libXi
        pango
        SDL2
        SDL2_image
        SDL2_ttf
        vulkan-loader
        vulkan-tools
        wayland
        wayland-protocols
        webkitgtk
    ]);
in with pkgs; mkShell {
    buildInputs = [
        cargo
        pkg-config
        rust-analyzer
        unstable.rustc
    ] ++ buildLibs;
    shellHook = ''
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${lib.makeLibraryPath buildLibs}"
        export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"
    '';
}

