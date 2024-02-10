# nix-store-veritysetup-generator

Like `systemd-veritysetup-generator` but for the Nix Store.

Reads the `storehash=` argument from the kernel commandline and dynamically
creates a service that sets up the verity protected device as
`dev/mapper/nix-store`.

If no `storehash=` argument on the kernel commandline is provided, nothing
happens.

Only works with the systemd initrd (`boot.initrd.systemd.enable = true;`).

Expects the UUIDs of the verity partitions to be set to the the first 128 bits
of the verity root hash for the data partition and the last 128 bits for the
hash partition. The easiest way to achieve this is by creating the partitions
with [`image.repart`](https://nixos.org/manual/nixos/stable/#sec-image-repart)
(which does the correct thing by default).

The get an idea of how to implement this, look at the
[test](./nix/tests/nix-store-veritysetup-generator.nix). An easier way
to build an appliance image that contains a verity protected Nix Store needs
a few more abstractions that I plan on building but that do not exist yet.

The plan is to eventually upstream this into Nixpkgs.
