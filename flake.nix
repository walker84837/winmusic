{
  description = "Environment for WinMusic bot";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system: let
    pkgs = nixpkgs.legacyPackages.${system};
  in rec {
    devShell = pkgs.mkShell {
      buildInputs = [
        pkgs.cmake
        pkgs.pkg-config
        pkgs.libopus
        pkgs.openssl
        pkgs.rustup
      ];

      shellHook = ''
        # Ensure rustup is set up and available
        if [ ! -d "$HOME/.cargo" ]; then
          rustup --version || echo "Rustup not installed!"
        fi
        export CARGO_TERM_COLOR=always
      '';
    };

    defaultPackage = pkgs.rustPackages.buildRustPackage rec {
      pname = "winmusic";
      version = "0.1.0";

      src = ./.;

      nativeBuildInputs = [
        pkgs.cmake
        pkgs.pkg-config
        pkgs.openssl
      ];

      buildInputs = [
        pkgs.libopus
        pkgs.openssl
      ];

      meta = with pkgs.stdenv.lib; {
        description = "Efficient and blazingly fast music bot written in Rust";
        license = licenses.mit;
      };
    };
  });
}
