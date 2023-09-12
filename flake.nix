{
  description = "Commercial DNP3 library by Step Function I/O (https://stepfunc.io/).";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/22.11";
    flake-utils.url = "github:numtide/flake-utils/v1.0.0";
  };

  outputs = inputs:
    with inputs;
    let f = system:
      let
        pname = "dnp3";
        version = "1.5.0-rc1";
        pkgs = import nixpkgs { inherit system; };
        package = let
          derivation = import ./derivation.nix;
          derivationArguments = { inherit pkgs pname version; };
        in derivation derivationArguments;
        devShell = let
          shell = import ./shell.nix;
          shellArguments = { inherit pkgs package; };
        in shell shellArguments;
      in {
        packages.default = package;
        apps.default = {
          type = "app";
          program = "${package}/bin/${pname}";
        };
        devShells.default = devShell;
      };
    in flake-utils.lib.eachSystem [ "aarch64-linux" "x86_64-linux" ] f;
}
