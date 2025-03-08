let
  unstable-pkgs = import (fetchTarball "https://channels.nixos.org/nixos-unstable/nixexprs.tar.xz") { };
in
{ pkgs ? import <nixpkgs> {} }:
let
    # fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") { };
    # rust-toolchain =
    #      fenix.default.toolchain;
  rust-toolchain =
      pkgs.symlinkJoin {
          name = "rust-toolchain";
          paths = with unstable-pkgs; [rustc cargo rustPlatform.rustcSrc clippy rustfmt gcc rust-analyzer];
      };
in
pkgs.mkShell rec {
    buildInputs = with pkgs;[
        openssl
        pkg-config
        cmake
        zlib
        bzip2
        lld
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
    RUST_SRC_PATH = "${unstable-pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    RUST_PATH="${rust-toolchain}";
    RUST_LOG="debug";
    LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
}
