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
      pkgs = nixpkgs.legacyPackages.${system};
      zigpkgs = zig.packages.${system};

      inherit (pkgs.stdenv) mkDerivation;
      inputs = [ zigpkgs.master ];

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
              cp zig-out/*/* $out/
            '';
            
            outputs = [ "out" ];
          };

      packages = {
        default = makePackage [];
        debug = makePackage ["-Doptimize=Debug"];
        release = makePackage ["-Doptimize=ReleaseFast"];
      };
    in
      {
        devShells.${system}.default = packages.debug;
        packages.${system} = packages;
      };
}