{ config, lib, pkgs, ... }:

let

  cfg = config.boot.initrd.systemd.nix-store-veritysetup-generator;

in

{

  options.boot.initrd.systemd.nix-store-veritysetup-generator = {

    enable = lib.mkEnableOption "nix-store-veritysetup-generator";

  };

  config = lib.mkIf cfg.enable {

    assertions = [
      {
        assertion = config.boot.initrd.systemd.enable;
        message = "nix-store-veritysetup-generator only works in the systemd initrd.";
      }
    ];

    boot.initrd.systemd = {

      contents = {
        "/etc/systemd/system-generators/nix-store-veritysetup-generator".source =
          "${pkgs.nix-store-veritysetup-generator}/bin/nix-store-veritysetup-generator";
      };

      storePaths = [
        "${config.boot.initrd.systemd.package}/bin/systemd-escape"
      ];

    };

  };

}
