{
  description = "A simple CLI tool to switch between Git identities per repository";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.writeShellApplication {
            name = "git-identity";
            runtimeInputs = [ pkgs.jq pkgs.fzf pkgs.git ];
            text = builtins.readFile ./git-identity.sh;
          };
        }
      );
    };
}
