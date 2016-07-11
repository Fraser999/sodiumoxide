#[macro_use]
extern crate unwrap;

#[cfg(not(feature = "get-libsodium"))]
extern crate pkg_config;

#[cfg(not(feature = "get-libsodium"))]
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

        unwrap!(pkg_config::find_library("libsodium"));

    }

}



#[cfg(feature = "get-libsodium")]
extern crate gcc;

#[cfg(all(windows, feature = "get-libsodium"))]
extern crate flate2;
#[cfg(all(windows, feature = "get-libsodium"))]
extern crate tar;

#[cfg(feature = "get-libsodium")]
fn get_install_dir() -> String {
    use std::env;
    unwrap!(env::var("OUT_DIR")) + "/installed"
}

#[cfg(feature = "get-libsodium")]
fn output_rustc_flags() {
    println!("cargo:rustc-link-lib=static=sodium");
    println!("cargo:rustc-link-search=native={}/lib", get_install_dir());
    println!("cargo:include={}/include", get_install_dir());
}

#[cfg(all(windows, feature = "get-libsodium"))]
fn main() {
    use std::fs::{self, File};
    use std::path::Path;
    use std::process::Command;
    use flate2::read::GzDecoder;
    use tar::Archive;

    const VERSION: &'static str = "1.0.10";

    if cfg!(target_env = "msvc") {
        panic!("This feature currently can't be used with MSVC builds.");
    }

    // Download gz tarball
    let basename = "libsodium-".to_string() + VERSION;
    let gz_filename = basename.clone() + "-mingw.tar.gz";
    let url = "https://download.libsodium.org/libsodium/releases/".to_string() + &gz_filename;
    let install_dir = get_install_dir();
    let gz_path = install_dir.clone() + &gz_filename;
    unwrap!(fs::create_dir_all(Path::new(&install_dir).join("lib")));

    let wget_output = Command::new("powershell")
        .arg("-Command")
        .arg("wget")
        .arg(&url)
        .arg("-OutFile")
        .arg(&gz_path)
        .output()
        .unwrap_or_else(|error| {
            panic!("Failed to run powershell wget command: {}", error);
        });
    if !wget_output.status.success() {
        panic!("\n{}\n{}\n",
               String::from_utf8_lossy(&wget_output.stdout),
               String::from_utf8_lossy(&wget_output.stderr));
    }

    // Unpack the tarball
    let gz_archive = unwrap!(File::open(&gz_path));
    let gz_decoder = unwrap!(GzDecoder::new(gz_archive));
    let mut archive = Archive::new(gz_decoder);

    // Extract just the appropriate version of libsodium.a and headers to the install path
    let arch_path = if cfg!(target_pointer_width = "32") {
        Path::new("libsodium-win32")
    } else if cfg!(target_pointer_width = "64") {
        Path::new("libsodium-win64")
    } else {
        panic!("target_pointer_width not 32 or 64")
    };

    let unpacked_include = arch_path.join("include");
    let unpacked_lib = arch_path.join("lib\\libsodium.a");
    let entries = unwrap!(archive.entries());
    for entry_result in entries {
        let mut entry = unwrap!(entry_result);
        let entry_path = unwrap!(entry.path()).to_path_buf();
        let full_install_path = if entry_path.starts_with(&unpacked_include) {
            let include_file = unwrap!(entry_path.strip_prefix(arch_path));
            Path::new(&install_dir).join(include_file)
        } else if entry_path == unpacked_lib {
            Path::new(&install_dir).join("lib").join("libsodium.a")
        } else {
            continue;
        };
        let _ = unwrap!(entry.unpack(full_install_path));
    }

    // Clean up
    let _ = fs::remove_file(gz_path);

    output_rustc_flags();
}



#[cfg(all(not(windows), feature = "get-libsodium"))]
fn main() {
    use std::env;
    use std::process::Command;

    let _ = Command::new("git").args(&["submodule", "update", "--init"]).status();
    let gcc = gcc::Config::new();
    let cc = format!("{}", gcc.get_compiler().path().display());
    let install_dir = get_install_dir();
    let prefix_arg = format!("--prefix={}", install_dir);
    let target = unwrap!(env::var("TARGET"));
    let host_arg = format!("--host={}", target);

    let configure_output = Command::new("./configure")
        .current_dir("libsodium")
        .env("CC", &cc)
        .arg(&prefix_arg)
        .arg(&host_arg)
        .arg("--enable-shared=no")
        .output()
        .unwrap_or_else(|error| {
            panic!("Failed to run './configure': {}", error);
        });
    if !configure_output.status.success() {
        panic!("\n{}\n{}\n",
               String::from_utf8_lossy(&configure_output.stdout),
               String::from_utf8_lossy(&configure_output.stderr));
    }

    let j_arg = format!("-j{}", unwrap!(env::var("NUM_JOBS")));
    let make_output = Command::new("make")
        .current_dir("libsodium")
        .env("V", "1")
        .arg("check")
        .arg(&j_arg)
        .output()
        .unwrap_or_else(|error| {
            panic!("Failed to run 'make check': {}", error);
        });
    if !make_output.status.success() {
        panic!("\n{}\n{}\n{}\n",
               String::from_utf8_lossy(&configure_output.stdout),
               String::from_utf8_lossy(&make_output.stdout),
               String::from_utf8_lossy(&make_output.stderr));
    }

    let install_output = Command::new("make")
        .current_dir("libsodium")
        .arg("install")
        .output()
        .unwrap_or_else(|error| {
            panic!("Failed to run 'make install': {}", error);
        });
    if !install_output.status.success() {
        panic!("\n{}\n{}\n{}\n{}\n",
               String::from_utf8_lossy(&configure_output.stdout),
               String::from_utf8_lossy(&make_output.stdout),
               String::from_utf8_lossy(&install_output.stdout),
               String::from_utf8_lossy(&install_output.stderr));
    }

    output_rustc_flags();
}
