{
  description = "Brillo - Control brightness of backlight and LED devices";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        # Build dependencies
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          makeWrapper
        ];

        # Build the brillo package
        brillo = pkgs.rustPlatform.buildRustPackage {
          pname = "brillo";
          version = "2.0.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs;

          # Create wrapper script to include ddcutil in PATH
          postInstall = ''
            wrapProgram $out/bin/brillo \
              --prefix PATH : "${pkgs.ddcutil}/bin"
          '';

          meta = with pkgs.lib; {
            description = "Control brightness of backlight, keyboard LEDs, and external monitors";
            homepage = "https://gitlab.com/CameronNemo/brillo";
            license = licenses.gpl3Only;
            maintainers = [ ];
            mainProgram = "brillo";
          };
        };
      in
      {
        # Default package
        packages.default = brillo;

        # Development shell
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            ddcutil
            rustToolchain
            rust-analyzer
            cargo-edit
            cargo-audit
            cargo-outdated
            clippy
            rustfmt
          ];

          shellHook = ''
            export RUST_LOG=info
            echo "Brillo development environment"
            echo "=============================="
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo ""
            echo "Commands:"
            echo "  cargo build     - Build the project"
            echo "  cargo run       - Run brillo"
            echo "  cargo test      - Run tests"
            echo "  cargo clippy    - Run linter"
            echo "  cargo fmt       - Format code"
            echo ""
            echo "Environment:"
            echo "  RUST_LOG=$RUST_LOG"
            echo ""
            echo "To run brillo: cargo run -- <args>"
          '';
        };

        # Apps for `nix run`
        apps.default = {
          type = "app";
          program = "${brillo}/bin/brillo";
        };

        # Legacy packages for compatibility
        legacyPackages.${system} = pkgs;
      }
    );
}
