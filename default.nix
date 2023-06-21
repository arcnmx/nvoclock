let
  lockData = builtins.fromJSON (builtins.readFile ./flake.lock);
  sourceInfo = lockData.nodes.std.locked;
  src = fetchTarball {
    url = "https://github.com/${sourceInfo.owner}/${sourceInfo.repo}/archive/${sourceInfo.rev}.tar.gz";
    sha256 = sourceInfo.narHash;
  };
in (import src).Flake.Bootstrap {
  path = ./.;
  inherit lockData;
  loadWith.defaultPackage = "nvoclock";
  fn = { outputs, system ? null, ... }: outputs // {
    windows = outputs.packages.nvoclock-w64;
    static = outputs.packages.nvoclock-static;
  };
}
