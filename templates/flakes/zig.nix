{
  inputs = {
    nixpkgs.url = github:NixOs/nixpkgs/nixos-23.05;
    zig.url     = github:mitchellh/zig-overlay;
  };

  outputs = { self, zig, nixpkgs }:
    let
      name = "REPLACEME";

      # environment
      system = "x86_64-linux";

      # project reqs
      inherit (pkgs) mkShell;
      inherit (pkgs.stdenv) mkDerivation;
      pkgs = nixpkgs.legacyPackages.${system};
      zigpkgs = zig.packages.${system};

      inputs = [ zigpkgs.master ];
      extraShellInputs = with pkgs; [ gdb wabt ];

      # developer shell
      shell = mkShell {
        packages = inputs ++ extraShellInputs;
      };

      # create a derivation for the build with some args for `zig build`
      makePackage = buildArgs:
        let
          argv =
            with pkgs.lib.strings;
            concatStrings (intersperse " " buildArgs);
        in
          mkDerivation {
            name = name;
            src = self;

            buildInputs = inputs;
            buildPhase = ''
              export HOME=$NIX_BUILD_TOP
              zig build ${argv}
            '';

            installPhase = ''
              mkdir -p $out/
              cp zig-out/lib/* $out/
            '';
          };

      packages = {
        default = makePackage [];
      };
    in
      {
        devShells.${system}.default = shell;
        packages.${system} = packages;
      };
}