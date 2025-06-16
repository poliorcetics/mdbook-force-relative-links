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

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    let
      perSystemOutputs = inputs.flake-utils.lib.eachDefaultSystem (import ./nix/per-system.nix inputs);
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
