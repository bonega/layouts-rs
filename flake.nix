{
  inputs = {
    flake-schemas.url = "https://flakehub.com/f/DeterminateSystems/flake-schemas/*";
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/*";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs =
    {
      self,
      flake-schemas,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [ "x86_64-linux" ];
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            };
          in
          f { inherit pkgs; }
        );
    in
    {
      schemas = flake-schemas.schemas;
      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default =
            let
              rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            in
            pkgs.mkShell {
              packages = with pkgs; [
                rustToolchain
                bacon
                taplo
                just
              ];
            };
        }
      );
    };
}
