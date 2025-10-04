{
  rustPlatform,
  pkgs,
}:
let
  version = "0.1.0";

  cli = rustPlatform.buildRustPackage {
    pname = "cactus-cli";
    inherit version;
    src = ./.;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };
    buildAndTestSubdir = "cactus-cli";
    cargoBuildFlags = [
      "-p"
      "cactus-cli"
    ];
  };

  gui = rustPlatform.buildRustPackage {
    pname = "cactus-gui";
    inherit version;
    src = ./.;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };
    buildAndTestSubdir = "cactus-gui";
    cargoBuildFlags = [
      "-p"
      "cactus-gui"
    ];
  };

  # Combine both packages
  all = pkgs.symlinkJoin {
    name = "cactus-all-${version}";
    paths = [
      cli
      gui
    ];
  };
in
{
  inherit cli gui all;
}
