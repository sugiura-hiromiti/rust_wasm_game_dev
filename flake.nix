{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        fx = fenix.packages.${system};

        # Build a toolchain explicitly:
        toolchain = fx.combine [
          fx.latest.rustc
          fx.latest.cargo
          fx.latest.rustfmt
          fx.latest.clippy
          fx.latest.rust-src
          # THIS is the missing piece: std for the wasm target
          fx.targets.wasm32-unknown-unknown.latest.rust-std
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.nodejs_24
            pkgs.wasm-pack
            toolchain
          ];

          shellHook = ''
            echo -e "\033[1;32m\nRust + WASM dev environment loaded"
            echo -e "System: ${system}"
            echo -e "wasm-pack: $(which wasm-pack 2>/dev/null || echo 'not found')"
            echo -e "rustc:     $(which rustc 2>/dev/null || echo 'not found')"
            echo -e "cargo:     $(which cargo 2>/dev/null || echo 'not found')\033[0m"
          '';
        };
      }
    );
}
