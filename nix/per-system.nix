{
  advisory-db,
  crane,
  nixpkgs,
  rust-overlay,
  self,
  treefmt-nix,
  ...
}:
system:
let
  overlays = [ (import rust-overlay) ];
  pkgs = import nixpkgs { inherit system overlays; };
  rust = import ./rust.nix { inherit advisory-db crane pkgs; };

  treefmtEval = import ./fmt.nix pkgs rust treefmt-nix;
in
{
  packages = rust.packages // {
    default = rust.packages.mdbook-force-relative-links;
  };

  checks = rust.checks // {
    formatting = treefmtEval.config.build.check self;
  };

  devShells.default = pkgs.mkShell {
    packages = [
      pkgs.nixfmt
      pkgs.mdbook
    ]
    ++ rust.extras.devShellPackages;
  };

  formatter = treefmtEval.config.build.wrapper;
}
