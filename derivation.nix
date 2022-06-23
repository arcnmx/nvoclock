{ rustPlatform
, windows
, nix-gitignore
, lib
, ...
}: with lib; let
  cargoToml = importTOML ./Cargo.toml;
in rustPlatform.buildRustPackage {
  pname = "nvoclock";
  version = cargoToml.package.version;

  src = nix-gitignore.gitignoreSourcePure [ ./.gitignore ''
    /.github
    /.git
    *.nix
  '' ] ./.;

  buildInputs = [
    windows.pthreads
  ];

  cargoSha256 = "sha256-tGnfK4yZ5HB6xH3gFn3gPRJV8mf+FkXx5WRIy6II2qU=";
  doCheck = false;
  meta = {
    platforms = platforms.windows;
  };
}
