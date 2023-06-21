{
  description = "NVIDIA overclocking CLI";
  inputs = {
    flakelib.url = "github:flakelib/fl";
    nixpkgs = { };
    rust = {
      url = "github:arcnmx/nixexprs-rust";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, flakelib, nixpkgs, rust, ... }@inputs: let
    nixlib = nixpkgs.lib;
  in flakelib {
    inherit inputs;
    systems = [ "x86_64-linux" "aarch64-linux" ];
    devShells = {
      plain = {
        mkShell, writeShellScriptBin, hostPlatform
      , enableRust ? true, cargo
      , rustTools ? [ ]
      , nativeBuildInputs ? [ ]
      }: mkShell {
        inherit rustTools;
        nativeBuildInputs = nativeBuildInputs
          ++ nixlib.optional enableRust cargo
          ++ nativeBuildInputs ++ [
            (writeShellScriptBin "generate" ''nix run .#generate "$@"'')
          ];
        RUST_LOG = "nvoclock=trace,nvapi*=trace";
      };
      stable = { rust'stable, outputs'devShells'plain }: outputs'devShells'plain.override {
        inherit (rust'stable) mkShell;
        enableRust = false;
      };
      dev = { rust'unstable, rust-w64-overlay, rust-w64, outputs'devShells'plain }: let
        channel = rust'unstable.override {
          channelOverlays = [ rust-w64-overlay ];
        };
      in outputs'devShells'plain.override {
        inherit (channel) mkShell;
        enableRust = false;
        rustTools = [ "rust-analyzer" ];
        nativeBuildInputs = [ rust-w64.pkgs.stdenv.cc.bintools ];
      };
      default = { outputs'devShells }: outputs'devShells.plain;
    };
    packages = {
      nvoclock = {
        __functor = _: import ./derivation.nix;
        fl'config.args = {
          crate.fallback = self.lib.crate;
        };
      };
      nvoclock-w64 = { pkgsCross'mingwW64, rust-w64, source }: pkgsCross'mingwW64.callPackage ./derivation.nix {
        inherit (rust-w64.latest) rustPlatform;
        inherit source;
      };
      nvoclock-static = { pkgsCross'musl64'pkgsStatic, source }: let
        rust = import inputs.rust { pkgs = pkgsCross'musl64'pkgsStatic; };
        nvoclock = pkgsCross'musl64'pkgsStatic.callPackage ./derivation.nix {
          inherit (rust.latest) rustPlatform;
          inherit source;
        };
      in nvoclock.overrideAttrs (old: {
        # XXX: why is this needed?
        NIX_LDFLAGS = old.NIX_LDFLAGS or "" + " -static";
        RUSTFLAGS = old.RUSTFLAGS or "" + " -C default-linker-libraries=yes";
      });
      default = { nvoclock }: nvoclock;
    };
    legacyPackages = { callPackageSet }: callPackageSet {
      source = { rust'builders }: rust'builders.wrapSource self.lib.crate.src;

      rust-w64 = { pkgsCross'mingwW64 }: import inputs.rust { inherit (pkgsCross'mingwW64) pkgs; };
      rust-w64-overlay = { rust-w64 }: let
        target = rust-w64.lib.rustTargetEnvironment {
          inherit (rust-w64) pkgs;
          rustcFlags = [ "-L native=${rust-w64.pkgs.windows.pthreads}/lib" ];
        };
      in cself: csuper: {
        sysroot-std = csuper.sysroot-std ++ [ cself.manifest.targets.${target.triple}.rust-std ];
        cargo-cc = csuper.cargo-cc // cself.context.rlib.cargoEnv {
          inherit target;
        };
        rustc-cc = csuper.rustc-cc // cself.context.rlib.rustcCcEnv {
          inherit target;
        };
      };

      generate = { rust'builders, outputHashes }: rust'builders.generateFiles {
        paths = {
          "lock.nix" = outputHashes;
        };
      };
      outputHashes = { rust'builders }: rust'builders.cargoOutputHashes {
        inherit (self.lib) crate;
      };
    } { };
    checks = {
    };
    lib = {
      crate = rust.lib.importCargo {
        path = ./Cargo.toml;
        inherit (import ./lock.nix) outputHashes;
      };
      inherit (self.lib.crate) version;
      releaseTag = "v${self.lib.version}";
    };
    config = {
      name = "nvoclock";
    };
  };
}
