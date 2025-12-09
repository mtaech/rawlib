use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    // For Windows MSVC
    if target.contains("msvc") {
        let lib_dir = PathBuf::from(&manifest_dir).join("libraw").join("msvc").join("lib");
        eprintln!("Using MSVC toolchain");
        eprintln!("Library directory: {}", lib_dir.display());
        
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=libraw_static");
        println!("cargo:rerun-if-changed=libraw/msvc/lib/libraw_static.lib");
        println!("cargo:rerun-if-changed=libraw/msvc/libraw/libraw.h");
    }
    // For Windows GNU (MinGW)
    else if target.contains("windows-gnu") {
        let lib_dir = PathBuf::from(&manifest_dir).join("libraw").join("lib");
        eprintln!("Using MinGW toolchain");
        eprintln!("Library directory: {}", lib_dir.display());
        
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=raw");
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rerun-if-changed=libraw/lib/libraw.a");
        println!("cargo:rerun-if-changed=libraw/libraw/libraw.h");
    }
    // For Linux/Mac
    else {
        let lib_dir = PathBuf::from(&manifest_dir).join("libraw").join("lib");
        eprintln!("Using Unix toolchain");
        eprintln!("Library directory: {}", lib_dir.display());
        
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=raw");
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rerun-if-changed=libraw/lib/libraw.a");
        println!("cargo:rerun-if-changed=libraw/libraw/libraw.h");
    }
    
    println!("cargo:rerun-if-changed=build.rs");
}
