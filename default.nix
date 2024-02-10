{ lib
, rustPlatform
, systemd
}:

rustPlatform.buildRustPackage {
  pname = "nix-store-veritysetup-generator";
  version = "0.1.0";

  src = ./rust;

  cargoLock = {
    lockFile = ./rust/Cargo.lock;
  };

  env = {
    SYSTEMD_VERITYSETUP_PATH = "${systemd}/lib/systemd/systemd-veritysetup";
    SYSTEMD_ESCAPE_PATH = "${systemd}/bin/systemd-escape";
  };

  meta = with lib; {
    description = "Systemd unit generator for a verity protected Nix Store";
    homepage = "https://github.com/nikstur/nix-store-veritysetup-generator";
    license = licenses.unlicense;
    maintainers = with lib.maintainers; [ nikstur ];
  };
}
