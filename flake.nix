{
  nixConfig = {
    substituters = [
      "https://cache.nixos.org/"
      "https://nix-community.cachix.org"
    ];
    trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";

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
            file = ./firmware/rust-toolchain.toml;
            hash = fakeHash;
          };

        nativeBuildInputs = with pkgs;
          [
            rustToolchain

            probe-rs-tools
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
