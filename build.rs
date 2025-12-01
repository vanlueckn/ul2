use std::env;
use std::path::{Path, PathBuf};

fn main() {
    // Try to find Ultralight libraries
    let lib_dir = find_ultralight_lib_dir();

    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // Link dynamically against required libraries
    println!("cargo:rustc-link-lib=dylib=AppCore");
    println!("cargo:rustc-link-lib=dylib=Ultralight");
    println!("cargo:rustc-link-lib=dylib=UltralightCore");
    println!("cargo:rustc-link-lib=dylib=WebCore");

    // Set the rpath so the libraries can be found at runtime
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| String::from("unknown"));
    if target_os == "linux" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    } else if target_os == "macos" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    }

    println!("cargo:rerun-if-changed={}", lib_dir.display());
    println!("cargo:rerun-if-env-changed=UL_DIR");
    println!("cargo:rerun-if-env-changed=ULTRALIGHT_DIR");
}

fn find_ultralight_lib_dir() -> PathBuf {
    // Get target platform to determine file extensions
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| String::from("unknown"));

    let lib_file_ext = match target_os.as_str() {
        "windows" => "lib",
        "macos" => "dylib",
        _ => "so", // Linux/others use .so
    };

    let lib_prefix = match target_os.as_str() {
        "windows" => "",
        _ => "lib", // Linux/others use lib as prefix
    };

    // Library files to search for
    let required_libs = [
        format!("{}Ultralight.{}", lib_prefix, lib_file_ext),
        format!("{}AppCore.{}", lib_prefix, lib_file_ext),
        format!("{}UltralightCore.{}", lib_prefix, lib_file_ext),
        format!("{}WebCore.{}", lib_prefix, lib_file_ext),
    ];

    println!("Ul path: {}",  env::var("UL_DIR"));

    // First check UL_DIR environment variable
    if let Ok(dir) = env::var("UL_DIR") {
        let path = PathBuf::from(dir);
        // Check directly in the provided path
        if has_required_libs(&path, &required_libs) {
            println!(
                "cargo:warning=Using Ultralight from UL_DIR: {}",
                path.display()
            );
            return path;
        }

        // Try lib subdirectory
        let lib_dir = path.join("lib");
        if lib_dir.exists() && has_required_libs(&lib_dir, &required_libs) {
            println!(
                "cargo:warning=Using Ultralight from UL_DIR/lib: {}",
                lib_dir.display()
            );
            return lib_dir;
        }

        // Try parent directory's lib (in case bin was specified)
        if let Some(parent) = path.parent() {
            let parent_lib = parent.join("lib");
            if parent_lib.exists() && has_required_libs(&parent_lib, &required_libs) {
                println!(
                    "cargo:warning=Using Ultralight from UL_DIR parent's lib: {}",
                    parent_lib.display()
                );
                return parent_lib;
            }
        }
    }

    // Also check ULTRALIGHT_DIR for compatibility
    if let Ok(dir) = env::var("ULTRALIGHT_DIR") {
        let path = PathBuf::from(dir);
        // Check directly in the provided path
        if has_required_libs(&path, &required_libs) {
            println!(
                "cargo:warning=Using Ultralight from ULTRALIGHT_DIR: {}",
                path.display()
            );
            return path;
        }

        // Try lib subdirectory
        let lib_dir = path.join("lib");
        if lib_dir.exists() && has_required_libs(&lib_dir, &required_libs) {
            println!(
                "cargo:warning=Using Ultralight from ULTRALIGHT_DIR/lib: {}",
                lib_dir.display()
            );
            return lib_dir;
        }
    }

    // Common library directories to check
    let common_dirs = [
        // Linux
        "/usr/local/lib",
        "/usr/lib",
        "/usr/local/ultralight/lib",
        "/usr/lib/ultralight/lib",
        // macOS
        "/usr/local/opt/ultralight/lib",
        "/opt/homebrew/opt/ultralight/lib",
        // Windows
        "C:\\Program Files\\Ultralight\\lib",
        "C:\\Ultralight\\lib",
        // Current directory
        "ultralight/lib",
        "./ultralight/lib",
        "../ultralight/lib",
        "lib",
        "./lib",
        "../lib",
        // Bin directory
        "bin",
        "./bin",
        "../bin",
    ];

    // Check each common directory
    for dir in common_dirs.iter() {
        let path = PathBuf::from(dir);
        if path.exists() && has_required_libs(&path, &required_libs) {
            println!(
                "cargo:warning=Found Ultralight libraries at: {}",
                path.display()
            );
            return path;
        }
    }

    // Check the parent of common directories (useful for bin->lib)
    for dir in &["bin", "./bin", "../bin"] {
        let path = PathBuf::from(dir);
        if let Some(parent) = path.parent() {
            let lib_dir = parent.join("lib");
            if lib_dir.exists() && has_required_libs(&lib_dir, &required_libs) {
                println!(
                    "cargo:warning=Found Ultralight libraries at: {}",
                    lib_dir.display()
                );
                return lib_dir;
            }
        }
    }

    // If we get here, we couldn't find Ultralight
    panic!(
        "Could not find Ultralight libraries. Please set UL_DIR environment variable to the directory containing the Ultralight libraries."
    );
}

fn has_required_libs(dir: &Path, libs: &[String]) -> bool {
    for lib in libs {
        if !dir.join(lib).exists() {
            return false;
        }
    }
    true
}
