pkgs: rust: treefmt-nix:
treefmt-nix.lib.evalModule pkgs {
  projectRootFile = "flake.nix";

  settings.global.excludes = [
    ".envrc"
  ];

  programs.mdformat.enable = true;
  programs.nixfmt.enable = true;
  programs.rustfmt = {
    enable = true;
    package = rust.extras.rustfmt;
  };
  programs.taplo.enable = true;
  programs.yamlfmt = {
    enable = true;
    settings = {
      continue_on_error = true;
      formatter = {
        type = "basic";
        eof_newline = true;
        retain_line_breaks_single = true;
        trim_trailing_whitespace = true;
      };
    };
  };
}
