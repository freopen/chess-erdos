{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux.extend fenix.overlays.default;
    in {
      packages.x86_64-linux.default = pkgs.runCommand "chess_erdos" {
        src = ./.;
        __noChroot = true;
        nativeBuildInputs = with pkgs; [
          binaryen
          cargo-make
          clang
          git
          nodePackages.pnpm
          pkg-config
          protobuf
          wasm-bindgen-cli
          wasm-pack
          (with fenix.packages."x86_64-linux";
            combine [
              minimal.rustc
              minimal.cargo
              targets.wasm32-unknown-unknown.latest.rust-std
            ])
        ];
        buildInputs = with pkgs; [ openssl curl zstd protobuf strace ];
        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        CURL_CA_BUNDLE = "/etc/ssl/certs/ca-bundle.crt";
        PROTOC = "${pkgs.protobuf}/bin/protoc";
      } ''
        cp -R $src/* .
        echo 'cafile = /etc/ssl/certs/ca-bundle.crt' > .npmrc
        export CARGO_HOME=$(mktemp -d cargo-home.XXX)
        export XDG_CACHE_HOME=$(pwd)/$(mktemp -d cache.XXX)
        cargo make release

        mkdir -p $out/bin
        cp target/release-server/chess-erdos $out/bin 
      '';
      devShells.x86_64-linux.default = pkgs.mkShell {
        inputsFrom = [ self.packages.x86_64-linux.default ];
        packages = (with pkgs; [ nil nixfmt ])
          ++ (with fenix.packages.x86_64-linux;
            [
              (combine [
                complete.cargo
                complete.clippy
                complete.rust-src
                complete.rustc
                complete.rustfmt
                targets.wasm32-unknown-unknown.latest.rust-std
              ])
            ]);
      };

      nixosModules.default = { config, pkgs, lib, ... }: {
        options.services.chess_erdos = {
          enable = lib.mkEnableOption "chess_erdos";
        };
        config = let
          opts = config.services.chess_erdos;
          pkg = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
        in lib.mkIf opts.enable {
          users.groups.chess_erdos = { };
          users.users.chess_erdos = {
            isSystemUser = true;
            group = "chess_erdos";
          };
          systemd.services.chess_erdos = {
            wantedBy = [ "multi-user.target" ];
            after = [ "network-online.target" ];
            wants = [ "network-online.target" ];
            startLimitIntervalSec = 0;
            path = with pkgs; [ curl zstd ];
            serviceConfig = {
              User = "chess_erdos";
              ExecStart = "${pkg}/bin/chess-erdos";
              WorkingDirectory = "/var/lib/chess_erdos";
              StateDirectory = "chess_erdos";
              StateDirectoryMode = "0700";
              Restart = "always";
              RestartSec = 60;
            };
          };
        };
      };
    };
}
