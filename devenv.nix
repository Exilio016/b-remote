{ pkgs, ... }:
let 
    dependencies = [
        pkgs.boost
        pkgs.postgresql
        pkgs.libpqxx
        pkgs.openssl
        pkgs.nssmdns
        pkgs.ncurses
    ];
in
{
    languages.cplusplus.enable = true;

    packages = dependencies;
    # Force Nix to export include paths into the environment variables
    env.CPATH = pkgs.lib.makeSearchPathOutput "dev" "include" dependencies;
    
    # For C projects, also expose it to the C preprocessor specifically
    env.C_INCLUDE_PATH = pkgs.lib.makeSearchPathOutput "dev" "include" dependencies;
    env.CPLUS_INCLUDE_PATH = pkgs.lib.makeSearchPathOutput "dev" "include" dependencies;

    env.LD_LIBRARY_PATH = "${pkgs.nssmdns}/lib";
}
