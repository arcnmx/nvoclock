{ pkgs ? import <nixpkgs> { } }: let
  rustPath = let
    local = builtins.tryEval <rust>;
    remote = builtins.fetchTarball {
      url = "https://github.com/arcnmx/nixexprs-rust/archive/master.tar.gz";
    };
  in if local.success then local.value else remote;
  inherit (pkgs.pkgsCross) mingwW64;
  rustW64 = import rustPath { inherit (mingwW64) pkgs; };
  rust = import rustPath { inherit pkgs; };
  nvoclock = mingwW64.callPackage ./derivation.nix {
    inherit (rustW64.stable) rustPlatform;
  };
  shell = rustW64.stable.mkShell {
    buildInputs = [
      mingwW64.windows.pthreads
    ];
    rustTools = [
      "rust-analyzer" "rust-src"
    ];
  };
in nvoclock // {
  inherit nvoclock shell;
}
