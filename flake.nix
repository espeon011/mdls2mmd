{
  description = "Rust environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
      in {
        devShells.default = pkgs.mkShell {
          name = "rust";
          nativeBuildInputs = [
            rustToolchain
          ];
          buildInputs = [];
          packages = [];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        packages.default = rustPlatform.buildRustPackage {
          pname = "mdls2mmd";
          version = "v1.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [];
          buildInputs = [];
          packages = [];
        };
      }
    );
}
