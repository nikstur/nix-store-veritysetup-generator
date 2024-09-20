{
  description = "Systemd unit generator for a verity protected Nix Store";

  inputs = {

    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    pre-commit-hooks-nix = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

  };

  outputs =
    inputs@{ self, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } (_: {

      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
        inputs.pre-commit-hooks-nix.flakeModule
      ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      flake.nixosModules = import ./nix/modules;

      perSystem =
        {
          config,
          system,
          pkgs,
          ...
        }:
        {

          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              (_final: _prev: {
                nix-store-veritysetup-generator = config.packages.nix-store-veritysetup-generator;
              })
            ];
          };

          packages = {
            nix-store-veritysetup-generator = pkgs.callPackage ./. { };
            default = config.packages.nix-store-veritysetup-generator;
          };

          checks = import ./nix/tests { inherit pkgs; };

          pre-commit = {
            check.enable = true;

            settings = {
              hooks = {
                nixfmt-rfc-style.enable = true;
                typos.enable = true;
              };
            };

          };

          devShells.default = pkgs.mkShell {
            shellHook = ''
              ${config.pre-commit.installationScript}
            '';

            packages = [
              pkgs.clippy
              pkgs.rustfmt
              pkgs.cargo-edit
            ];

            inputsFrom = [ config.packages.nix-store-veritysetup-generator ];

            # Use a fake path so that the test does not depend on specific Nix
            # store paths.
            SYSTEMD_VERITYSETUP_PATH = "systemd-veritysetup";
            SYSTEMD_ESCAPE_PATH = "${pkgs.systemd}/bin/systemd-escape";

            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          };

        };
    });
}
