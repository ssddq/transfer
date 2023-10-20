{
  inputs =
  { nixpkgs.url = "github:NixOS/nixpkgs";

    release =
    { flake = false;
      url = "https://github.com/ssddq/transfer/releases/download/binary/transfer";
    };
  };

  outputs = { self, nixpkgs, release }:
  let system = "x86_64-linux";

      pkgs = nixpkgs.legacyPackages.${system};

  in
  { packages.${system}.default = pkgs.stdenv.mkDerivation
    { name = "transfer";
      version = "0.1";

      src = ./.;

      nativeBuildInputs = [ pkgs.autoPatchelfHook ];

      buildInputs = with pkgs;
      [ libgcc
      ];

      installPhase =
      ''
        install -m755 -D ${release.outPath} $out/bin/transfer
      '';
    };
    shell = pkgs.mkShell
    { buildInputs =
      [ pkgs.cargo
        pkgs.rustc
      ];
    };

    devShells.${system} =
    { default = self.shell;
    };

  };

}
