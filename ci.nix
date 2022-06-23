{ config, pkgs, lib, ... }: with pkgs; with lib; let
  nvoclock = import ./. { inherit pkgs; };
  artifactRoot = ".ci/artifacts";
  artifacts = "${artifactRoot}/bin/nvoclock*";
in {
  config = {
    name = "nvoclock";
    ci.gh-actions.enable = true;
    cache.cachix.arc.enable = true;
    channels = {
      nixpkgs = {
        # see https://github.com/arcnmx/nixexprs-rust/issues/10
        args.config.checkMetaRecursively = false;
      };
      rust = "master";
    };
    tasks = {
      build.inputs = singleton (nvoclock.nvoclock.overrideAttrs (old: {
        meta = old.meta // {
          # workaround for ci bug
          platforms = platforms.unix ++ old.meta.platforms or [];
        };
      }));
    };

    artifactPackage = nvoclock.nvoclock;

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
              uses.path = "actions/upload-artifact@v2";
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
  options = {
    artifactPackage = mkOption {
      type = types.package;
    };
  };
}
