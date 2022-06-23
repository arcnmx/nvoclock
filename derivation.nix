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

  cargoSha256 = "sha256-3+e/PvXV4NO+si8lt0X9GfDw9S8me3iYT71WysT9LoY=";
  doCheck = false;
  meta = {
    platforms = platforms.windows;
  };
}
