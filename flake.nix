{
  description = "Alternative client for u.gg";

  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
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
            cargo-edit
            cargo-features-manager
            cargo-tauri
            nodejs_22
            rust
          ];
        };
      }
    );
}
