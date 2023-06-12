{
  description = "a project configurator";
  
  inputs = {
    nixpkgs.url = github:nixos/nixpkgs/release-23.05;
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, naersk }:
    let
      system = "x86_64-linux";
      name = "make-project";

      pkgs = (import nixpkgs) {
        inherit system;
      };
      naersk' = pkgs.callPackage naersk {};

      mkPackage = release: naersk'.buildPackage {
        inherit name release;
        src = ./.;

        # naersk builds the executable at $out/bin/${name}. this will move it to
        # $out and copy templates over
        postInstall = ''
          install $out/bin/${name} $out/
          rm -r $out/bin
          cp -r templates/ $out/
        '';
      };

      projectPkgs = {
        debug = mkPackage false;
        release = mkPackage true;
      };
    in {
      packages.${system} = {
        default = projectPkgs.release;
        inherit (projectPkgs) debug release;
      };
    };
}
