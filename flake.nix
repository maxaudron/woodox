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

    crane.url = "github:ipetkov/crane";
    parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      fenix,
      crane,
      parts,
    }:
    parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      perSystem =
        {
          config,
          lib,
          self',
          inputs',
          pkgs,
          system,
          ...
        }:
        let
          toolchain =
            with fenix.packages.${system};
            combine [
              stable.toolchain
              targets.thumbv6m-none-eabi.stable.rust-std
              targets."thumbv8m.main-none-eabihf".stable.rust-std
              targets.riscv32imac-unknown-none-elf.stable.rust-std
            ];

          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

          fileSetForCrate =
            crate:
            lib.fileset.toSource {
              root = ./firmware;
              fileset = lib.fileset.unions [
                ./firmware/Cargo.toml
                ./firmware/Cargo.lock
                (craneLib.fileset.commonCargoSources ./firmware/library)
                (craneLib.fileset.commonCargoSources crate)
              ];
            };
        in
        {
          devShells.default = craneLib.devShell {
            checks = self.checks.${system};

            DEFMT_LOG = "info";

            packages = with pkgs; [
              probe-rs-tools
              flip-link
              elf2uf2-rs
              gcc-arm-embedded
              cargo-binutils
              picotool

              fenix.packages.${system}.rust-analyzer
            ];
          };
        };
    };
}
