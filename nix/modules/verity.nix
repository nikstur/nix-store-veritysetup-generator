{ config, lib, ... }:

let

  cfg = config.boot.initrd.systemd.verity;

in

{

  options.boot.initrd.systemd.verity = {

    enable = lib.mkEnableOption "verity";

  };

  config = lib.mkIf cfg.enable {

    boot.initrd = {

      availableKernelModules = [
        "dm_mod"
        "dm_verity"
      ];

      # We need LVM for dm-verity to work.
      services.lvm.enable = true;

      systemd = {

        additionalUpstreamUnits = [
          "veritysetup-pre.target"
          "veritysetup.target"
          "remote-veritysetup.target"
        ];

        storePaths = [
          "${config.boot.initrd.systemd.package}/lib/systemd/systemd-veritysetup"
          "${config.boot.initrd.systemd.package}/lib/systemd/system-generators/systemd-veritysetup-generator"
        ];

      };
    };

  };

}
