#[cfg(not(feature = "build-libsodium"))]
extern crate pkg_config;

#[cfg(not(feature = "build-libsodium"))]
fn main() {
    use std::env;

    if let Ok(lib_dir) = env::var("SODIUM_LIB_DIR") {

        println!("cargo:rustc-flags=-L native={}", lib_dir);

        let mode = match env::var_os("SODIUM_STATIC") {
            Some(_) => "static",
            None => "dylib",
        };
        println!("cargo:rustc-flags=-l {0}=sodium", mode);

    } else {

        pkg_config::find_library("libsodium").unwrap();

    }

}



#[cfg(feature = "build-libsodium")]
extern crate gcc;

#[cfg(feature = "build-libsodium")]
fn main() {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let mut gcc_config = gcc::Config::new();
    let _ = gcc_config.include("libsodium/src/libsodium/include/sodium");

    let file = File::open("sources.txt").unwrap();
    let file_reader = BufReader::new(file);

    for line in file_reader.lines() {
        let _ = gcc_config.file(line.unwrap());
    }
    let _ = gcc_config.compile("libsodium.a");
}
