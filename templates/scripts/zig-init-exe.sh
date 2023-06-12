set -xe

# this will use the flake's zig, so should work for any zig with 'init-exe'
nix develop \
    -c bash \
    -c 'zig init-exe'