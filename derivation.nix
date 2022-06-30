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

  cargoSha256 = "sha256-eJR3fDlCQd3+N7bTfVTTe5PV7c4WLYgirl9LUR23AiM=";
  doCheck = false;
  meta = {
    platforms = platforms.windows;
  };
}
