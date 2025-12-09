fn main() {
    // Link against LibRaw static library
    println!("cargo:rustc-link-search=native=libraw/lib");
    println!("cargo:rustc-link-lib=static=raw");
    
    // Tell Cargo to rerun if the library changes
    println!("cargo:rerun-if-changed=libraw/lib/libraw.a");
    println!("cargo:rerun-if-changed=libraw/libraw/libraw.h");
    
    // For Windows, you might also need to link against C++ standard library
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
    
    // For Linux/Mac
    #[cfg(not(target_os = "windows"))]
    {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
}
