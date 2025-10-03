{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.minimal.override {
          extensions = [
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
        };
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        cactus = pkgs.callPackage ./default.nix { inherit rustPlatform; };
      in
      {
        packages = {
          default = cactus.all;
          cli = cactus.cli;
          gui = cactus.gui;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Dependencies
            rustToolchain
            just

            # Testing materials
            # Engines
            stockfish
            lc0
            # other runners
            fastchess
            cutechess
          ];
        };
      }
    );
}
