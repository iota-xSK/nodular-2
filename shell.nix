{ pkgs ? import <nixpkgs> {}, lib ? pkgs.lib }: 

pkgs.mkShell {
  buildInputs = [
    pkgs.rustup
    pkgs.cmake
    pkgs.clang
    pkgs.pkg-config
    pkgs.wayland
    pkgs.glfw
    pkgs.libGL
    pkgs.xorg.libXrandr
    pkgs.xorg.libXinerama
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.gtk4
    pkgs.gtk3
    pkgs.alsa-lib
  ];

  # Set environment variables for library paths
  LD_LIBRARY_PATH = lib.makeLibraryPath [
    pkgs.libGL
    pkgs.xorg.libXrandr
    pkgs.xorg.libXinerama
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.glib
    pkgs.gtk4
    pkgs.gtk3
    pkgs.alsa-lib
  ];

  LIBCLANG_PATH = "${pkgs.llvmPackages_16.libclang.lib}/lib";
}

