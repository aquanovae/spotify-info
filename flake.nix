{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = { flake-parts, ... }@inputs: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ "x86_64-linux" ];

    perSystem = { pkgs, ... }: with pkgs; let 

      buildInputs = [
        openssl
      ];

      nativeBuildInputs = [
        cargo
        cargo-edit
        pkg-config
        rustc
      ];

    in {

      devShells.default = mkShell {
        name = "rust";
        inherit buildInputs nativeBuildInputs;
      };

      packages.default = rustPlatform.buildRustPackage {
        pname = "spotify-info";
        version = "0.1.0";
        src = ./.;
        cargoHash = "sha256-HAJ+U2KMiNVp7ud9wZ3tDvonD2XgvE3Xi2mvvG7Yi3k=";
        inherit buildInputs nativeBuildInputs;
      };
    };
  };
}
