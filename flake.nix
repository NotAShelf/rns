{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";

  outputs = {nixpkgs, ...}: let
    inherit (nixpkgs) legacyPackages lib;
    forAllSystems = lib.genAttrs ["x86_64-linux"];
  in {
    packages = forAllSystems (system: let
      pkgs = legacyPackages.${system};
      fs = lib.fileset;
    in {
      default = pkgs.rustPlatform.buildRustPackage {
        pname = "rns";
        version = "0.1.1";
        src = fs.toSource {
          root = ./.;
          fileset = fs.unions [
            (fs.fileFilter (file: builtins.any file.hasExt ["rs"]) ./.)
            ./Cargo.toml
            ./Cargo.lock
          ];
        };

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        nativeBuildInputs = [
          pkgs.pkg-config
        ];

        buildInputs = [
          pkgs.luajit
        ];

        postInstall = ''
          mkdir -p $out/include
          install -m755 -D ${./include/rns.h} $out/include/rns.h
        '';

        meta = {
          description = "Neovim configuration in Rust, C, or Zig";
          homepage = "https://github.com/NotAShelf/rns";
          license = lib.licenses.mpl20;
          maintainers = [lib.maintainers.NotAShelf];
        };
      };
    });

    devShells = forAllSystems (system: let
      pkgs = legacyPackages.${system};
    in {
      default = pkgs.mkShell {
        name = "rns-dev";
        packages = with pkgs; [
          cargo
          clippy

          # Required to build RNS
          luajit

          rust-analyzer-unwrapped
          (rustfmt.override {asNightly = true;})
        ];
      };
    });

    formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.alejandra);
  };
}
