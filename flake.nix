{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = { flake-parts, ... }@inputs: (
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];

      perSystem = { pkgs, ... }: with pkgs; let 
        buildInputs = [
          openssl
        ];
        nativeBuildInputs = [
          cargo
          cargo-edit
          rustc
          pkg-config
        ];
      in {
        packages.spotify-info = rustPlatform.buildRustPackage {
          inherit buildInputs nativeBuildInputs;
          pname = "spotify-info";
          version = "0.1.0";
          src = ./.;
          cargoHash = "sha256-czHT98gNOVFd+kGBmfr9QUL4Hw0l3qBIFYkQmkhfIAY=";
        };

        devShells.default = mkShell {
          inherit buildInputs nativeBuildInputs;
          name = "rust";
        };
      };
    }
  );
}
