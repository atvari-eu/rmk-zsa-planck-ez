{
  description = "RMK firmware for the ZSA Planck EZ";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    # nixpkgs' rustc ships only host std; the firmware needs the bare-metal
    # thumbv7em-none-eabihf target and llvm-tools for rust-objcopy.
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    {
      # Enter with `nix develop`
      devShells = builtins.mapAttrs (
        system: pkgs':
        let
          pkgs = pkgs'.extend inputs.rust-overlay.overlays.default;
          rust = pkgs.rust-bin.stable.latest.default.override {
            targets = [ "thumbv7em-none-eabihf" ];
            extensions = [
              "clippy"
              "llvm-tools"
              "rust-analyzer"
              "rust-src"
              "rustfmt"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = [
              rust
              pkgs.flip-link
              pkgs.cargo-binutils
              pkgs.probe-rs-tools
              pkgs.dfu-util
              # build.rs compresses vial.json through the xz2 crate.
              pkgs.pkg-config
              pkgs.xz
              pkgs.git
            ];
          };
        }
      ) inputs.nixpkgs.legacyPackages;
    };
}
