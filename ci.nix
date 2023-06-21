{ config, pkgs, lib, ... }: with pkgs; with lib; let
  nvoclock = import ./. { inherit pkgs; };
  inherit (nvoclock) checks packages;
  artifactRoot = ".ci/artifacts";
  artifacts = "${artifactRoot}/bin/nvoclock*";
  nvoclock-checked = (packages.nvoclock.override {
    buildType = "debug";
  }).overrideAttrs (_: {
    doCheck = true;
  });
in {
  config = {
    name = "nvoclock";
    ci = {
      version = "v0.6";
      gh-actions.enable = true;
    };
    cache.cachix.arc.enable = true;
    channels = {
      nixpkgs = {
        # see https://github.com/arcnmx/nixexprs-rust/issues/10
        args.config.checkMetaRecursively = false;
        version = "23.05";
      };
    };
    tasks = {
      build.inputs = singleton nvoclock-checked;
      build-windows.inputs = singleton packages.nvoclock-w64;
      build-static.inputs = singleton packages.nvoclock-static;
    };
    artifactPackages = {
      musl64 = packages.nvoclock-static;
      win64 = packages.nvoclock-w64;
    };

    artifactPackage = runCommand "nvoclock-artifacts" { } (''
      mkdir -p $out/bin
    '' + concatStringsSep "\n" (mapAttrsToList (key: nvoclock: ''
        ln -s ${nvoclock}/bin/nvoclock${nvoclock.stdenv.hostPlatform.extensions.executable} $out/bin/nvoclock-${key}${nvoclock.stdenv.hostPlatform.extensions.executable}
    '') config.artifactPackages));

    gh-actions = {
      jobs = {
        ${config.id} = {
          permissions = {
            contents = "write";
          };
          step = {
            artifact-build = {
              order = 1100;
              name = "artifact build";
              uses = {
                # XXX: a very hacky way of getting the runner
                inherit (config.gh-actions.jobs.${config.id}.step.ci-setup.uses) owner repo version;
                path = "actions/nix/build";
              };
              "with" = {
                file = "<ci>";
                attrs = "config.artifactPackage";
                out-link = artifactRoot;
              };
            };
            artifact-upload = {
              order = 1110;
              name = "artifact upload";
              uses.path = "actions/upload-artifact@v3";
              "with" = {
                name = "nvoclock";
                path = artifacts;
              };
            };
            release-upload = {
              order = 1111;
              name = "release";
              "if" = "startsWith(github.ref, 'refs/tags/')";
              uses.path = "softprops/action-gh-release@v1";
              "with".files = artifacts;
            };
          };
        };
      };
    };
  };
  options = with types; {
    artifactPackage = mkOption {
      type = package;
    };
    artifactPackages = mkOption {
      type = attrsOf package;
    };
  };
}
