{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    crane.url = "github:ipetkov/crane";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      advisory-db,
      crane,
      rust-overlay,
      ...
    }:
    let
      perSystemOutputs = flake-utils.lib.eachDefaultSystem (
        system:
        import ./nix/per-system.nix {
          inherit
            advisory-db
            crane
            nixpkgs
            rust-overlay
            system
            ;
        }
      );
    in
    perSystemOutputs
    // {
      overlays.default = (
        final: _prev: {
          mdbook-force-relative-links =
            perSystemOutputs.packages.${final.stdenv.system}.mdbook-force-relative-links;
        }
      );
    };
}
