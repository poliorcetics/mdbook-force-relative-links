{
  advisory-db,
  crane,
  nixpkgs,
  rust-overlay,
  system,
}:
let
  overlays = [ (import rust-overlay) ];
  pkgs = import nixpkgs { inherit system overlays; };
  rust = import ./rust.nix { inherit advisory-db crane pkgs; };
in
{
  packages = rust.packages // {
    default = rust.packages.mdbook-force-relative-links;
  };

  checks = rust.checks // {
    ## Nix ##

    nix-fmt-checks = pkgs.stdenv.mkDerivation {
      name = "nix-fmt-checks";
      src = ./..;
      dontBuild = true;
      nativeBuildInputs = [ pkgs.nixfmt-rfc-style ];
      doCheck = true;
      checkPhase = ''
        nixfmt --check .
      '';
      installPhase = ''
        mkdir "$out"
      '';
    };
  };

  devShells.default = pkgs.mkShell {
    packages = [ pkgs.nixfmt-rfc-style ] ++ rust.extras.devShellPackages;
  };

  formatter = nixpkgs.legacyPackages.${system}.nixfmt-rfc-style;
}
