{
  description = "...";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
    alejandra.url = "github:kamadorueda/alejandra/3.0.0";
    alejandra.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crate2nix,
    alejandra,
    ...
  }: let
    name = "mops";
  in
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        # Overlay some stuff to secure the versions we want
        overlays = [
          (import rust-overlay)
          (self: super: {
            # rustc = super.latest.rustChannels.stable.rust;
            # inherit (super.latest.rustChannels.stable) cargo rust rust-fmt rust-std clippy;
            rustc = self.rust-bin.stable.latest.default.override {extensions = ["llvm-tools-preview"];};
            cargo = self.rust-bin.stable.latest.default.override {extensions = ["llvm-tools-preview"];};
          })
        ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Configuration for the non-Rust dependencies
        buildInputs = with pkgs; [] ++ pkgs.lib.optionals pkgs.stdenv.targetPlatform.isDarwin [pkgs.darwin.apple_sdk.frameworks.Security];
        nativeBuildInputs = with pkgs;
          [
            pkg-config
            openssl
            
            # For Rust
            rustc
            cargo

            # For coverage checks & test reports
            grcov
            cargo2junit

            # Bloat checks
            cargo-bloat

            # Nix tooling
            rnix-lsp

            # For Prusti
            # jdk11
            # rustup

            # Compliance checks
            cargo-deny

            # misc tooling
            jq
            skopeo
            cocogitto
            cargo-edit
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.targetPlatform.isDarwin [pkgs.darwin.apple_sdk.frameworks.Security];

        buildEnvVars = {
        };
      in {

        # `nix develop`
        devShell =
          pkgs.mkShell
          {
            inherit buildInputs nativeBuildInputs;
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          }
          // buildEnvVars;

        formatter = alejandra.defaultPackage.${system};
      }
    );
}
