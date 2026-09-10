{
  lib,
  rustPlatform,
  systemd,
}:

let
  cargoToml = fromTOML (builtins.readFile ./rust/Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  inherit (cargoToml.package) version;

  src = ./rust;

  cargoLock = {
    lockFile = ./rust/Cargo.lock;
  };

  env = {
    SYSTEMD_VERITYSETUP_PATH = "${systemd}/lib/systemd/systemd-veritysetup";
    SYSTEMD_ESCAPE_PATH = "${systemd}/bin/systemd-escape";
  };

  # Use a fake path in tests so that they are not dependent on specific Nix
  # Store paths and thus don't break on different Nixpkgs invocations. This is
  # relevant so that this package can be compiled on different architectures.
  preCheck = ''
    export SYSTEMD_VERITYSETUP_PATH="systemd-veritysetup";
  '';

  stripAllList = [ "bin" ];

  meta = with lib; {
    description = "Systemd unit generator for a verity protected Nix Store";
    homepage = "https://github.com/nikstur/nix-store-veritysetup-generator";
    license = licenses.mit;
    maintainers = with lib.maintainers; [ nikstur ];
  };
}
