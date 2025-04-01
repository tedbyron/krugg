{
  description = "League of Legends build helper";

  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    flake-utils.url = "flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      with pkgs;
      {
        formatter = nixfmt-rfc-style;

        devShells.default = mkShellNoCC {
          packages = [
            nodejs_22
            rust
          ];
        };
      }
    );
}
