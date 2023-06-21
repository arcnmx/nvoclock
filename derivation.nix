let
  self = import ./. { pkgs = null; system = null; };
in {
  rustPlatform
, lib
, buildType ? "release"
, cargoLock ? crate.cargoLock
, source ? crate.src
, crate ? self.lib.crate
}: with lib; rustPlatform.buildRustPackage {
  pname = crate.name;
  inherit (crate) version;

  src = source;
  inherit cargoLock buildType;
  doCheck = false;

  meta = {
    description = "NVIDIA overclocking CLI";
    homepage = "https://github.com/arcnmx/nvoclock";
    license = licenses.mit;
    maintainers = [ maintainers.arcnmx ];
    platforms = platforms.linux ++ platforms.windows;
    mainProgram = "nvoclock";
  };
}
