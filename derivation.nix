{ pkgs
, pname
, version
}:

pkgs.rustPlatform.buildRustPackage rec {
  inherit pname version;
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
}
