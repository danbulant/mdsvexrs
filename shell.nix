{ pkgs ? import <nixpkgs> {}, pkgsun ? import <nixos-unstable> {} }:
let
    fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") { };
    rust-toolchain =
         fenix.default.toolchain;
in
pkgs.mkShell rec {
    buildInputs = with pkgs;[
        openssl
        pkg-config
        cmake
        zlib
        rust-toolchain

        libclang
        llvmPackages.clang
        openssl
        clang
        llvmPackages.libclang.lib stdenv.cc.libc
    ];
    nativeBuildInputs = with pkgs; [
        pkg-config
        fontconfig
        rustPlatform.bindgenHook
    ];
    
  preConfigure = with pkgs;''
    export BINDGEN_EXTRA_CLANG_ARGS="-isystem ${clang}/resource-root/include $NIX_CFLAGS_COMPILE"
  '';

    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
    OPENSSL_DIR="${pkgs.openssl.dev}";
    OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib";
    RUST_SRC_PATH = "${pkgsun.rust.packages.stable.rustPlatform.rustLibSrc}";
    RUST_PATH="${rust-toolchain}";
    RUST_LOG="debug";
    LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
}
