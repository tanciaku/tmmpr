{
  description = "terminal mind mapper";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "tmmpr";
          version = "0.1.1";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          meta = {
            description = "terminal mind mapper";
            homepage = "https://github.com/tanciaku/tmmpr";
            license = pkgs.lib.licenses.mit;
            maintainers = [];
            mainProgram = "tmmpr";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
          ];
        };
      });
}