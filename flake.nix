{
  nixConfig = {
    substituters = [
      "https://cache.nixos.org/"
      "https://nix-community.cachix.org"
      "https://nix.cache.vapor.systems"
    ];
    trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "nix.cache.vapor.systems-1:OjV+eZuOK+im1n8tuwHdT+9hkQVoJORdX96FvWcMABk="
    ];
  };

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-22.11";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, fenix, crane }:
    with nixpkgs.lib;
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        rustToolchain = with fenix.packages.${system};
          fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-S4dA7ne2IpFHG+EnjXfogmqwGyDFSRWFnJ8cy4KZr1k=";
          };

        nativeBuildInputs = with pkgs;
          [
            rustToolchain

            cargo-embed
            probe-run
            flip-link
            elf2uf2-rs
            gcc-arm-embedded
            cargo-binutils
          ];

        # CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        # CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        CARGO_BUILD_TARGET = null;
        CARGO_BUILD_RUSTFLAGS = null;

        # crane setup
        craneLib = crane.lib.${system}.overrideToolchain rustToolchain;
        src = craneLib.cleanCargoSource ./.;

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src;
          inherit nativeBuildInputs CARGO_BUILD_TARGET CARGO_BUILD_RUSTFLAGS;
        };
      in {
        packages = { };

        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs CARGO_BUILD_TARGET CARGO_BUILD_RUSTFLAGS;
        };

        formatter = pkgs.nixfmt;
      });
}
