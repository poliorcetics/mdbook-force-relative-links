# Rust packages, checks and extras for `../flake.nix`.
#
# The intent is to need as little extras as possible and only expose the absolute minimum, to allow
# for easier maintenance.
{
  advisory-db,
  crane,
  pkgs,
}:
let
  baseRustToolchain = pkgs.rust-bin.stable.latest;
  extensions = [
    "rust-src"
    "cargo"
    "rustc"
  ];
  rustBuildToolchain = baseRustToolchain.default.override { inherit extensions; };
  # rust-analyzer is only needed in the devShell, let's not download it for nothing all the time
  rustDevToolchain = baseRustToolchain.default.override {
    extensions = extensions ++ [ "rust-analyzer" ];
  };
  # Nightly rustfmt must be higher prio than the normal one since cargo searches by path
  # Also, it's only necessary in the devShell and fmt check
  rustfmt = (pkgs.lib.hiPrio pkgs.rust-bin.nightly.latest.rustfmt);

  nativeBuildInputs = [ rustBuildToolchain ];

  buildInputs =
    with pkgs;
    lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.SystemConfiguration
      darwin.apple_sdk.frameworks.CoreFoundation
    ];

  craneLib = (crane.mkLib pkgs).overrideToolchain rustBuildToolchain;

  # Filtering the source leads to a smaller footprint in the nix store and ensure we don't
  # use unexpected files during builds
  src = pkgs.lib.cleanSourceWith {
    src = craneLib.path ./..; # The original, unfiltered source
    filter = path: type: (craneLib.filterCargoSources path type);
  };

  commonArgs = {
    inherit buildInputs nativeBuildInputs src;
    pname = "mdbook-force-relative-links";

    cargoExtraArgs = "--offline --locked";

    doCompressAndInstallFullArchive = false;
    dontStrip = true;
    separateDebugInfo = true;
    strictDeps = true;
  };

  # Vendoring the deps first avoids redownloading them for nothing
  cargoVendoredDeps = craneLib.vendorMultipleCargoDeps { cargoLockList = [ ./../Cargo.lock ]; };

  # Building artifacts separately allows caching them separately too
  cargoArtifacts = craneLib.buildDepsOnly (commonArgs // { cargoVendorDir = cargoVendoredDeps; });

  packages = {
    mdbook-force-relative-links = craneLib.buildPackage (
      commonArgs
      // {
        inherit cargoArtifacts;
        doCheck = false;
      }
    );
  };
in
{
  inherit packages;

  extras = {
    devShellPackages = [
      rustfmt
      rustDevToolchain
    ] ++ buildInputs ++ nativeBuildInputs;
  };

  checks = packages // {
    # Audit dependencies
    mdbook-force-relative-links-audit = craneLib.cargoAudit { inherit src advisory-db; };

    # Run clippy in checks
    mdbook-force-relative-links-clippy = craneLib.cargoClippy (
      commonArgs
      // {
        cargoArtifacts = cargoArtifacts;
        cargoClippyExtraArgs = "--tests --workspace -- --deny warnings";
      }
    );

    # Check formatting
    mdbook-force-relative-links-fmt = pkgs.writeShellApplication {
      name = "mdbook-force-relative-links-fmt-check";
      runtimeInputs = [
        rustfmt
        rustBuildToolchain
      ];
      text = ''
        cargo fmt --version
        cargo fmt --check --verbose
      '';
    };

    mdbook-force-relative-links-nextest = craneLib.cargoNextest (
      commonArgs
      // {
        inherit cargoArtifacts;
        CARGO_TERM_COLOR = "always";
        NEXTEST_FAILURE_OUTPUT = "final";
        NEXTEST_HIDE_PROGRESS_BAR = "1";
        NEXTEST_TEST_THREADS = 1;
        cargoNextestExtraArgs = "--no-fail-fast --workspace";
      }
    );
  };
}
