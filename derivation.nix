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

  cargoSha256 = "sha256-BLUTQYFTKP4vL3jwFR7PmYZU1fwGomgkjbkjj3gVPyo=";
  doCheck = false;
  meta = {
    platforms = platforms.windows;
  };
}
