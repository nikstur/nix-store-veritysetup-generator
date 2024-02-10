{ pkgs, ... }:

let
  runTest = module: pkgs.testers.runNixOSTest {
    imports = [ module ];
    globalTimeout = 5 * 60;
    extraBaseModules = {
      imports = builtins.attrValues (import ../modules);
    };
  };
in
{
  nix-store-veritysetup-generator = runTest ./nix-store-veritysetup-generator.nix;
}
