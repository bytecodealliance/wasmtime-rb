 {
    inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    inputs.flake-utils.url = "github:numtide/flake-utils";

    outputs = { nixpkgs, flake-utils, ... }:
      flake-utils.lib.eachDefaultSystem (system: {
        devShells.default = nixpkgs.legacyPackages.${system}.mkShell {
          packages = with nixpkgs.legacyPackages.${system}; [
            ruby # Stable
            bundler
            libffi
            llvmPackages_latest.libcxx
          ];
        };
      });
  }
