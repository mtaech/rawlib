//! Build script for RawLib - automatically detects and links appropriate LibRaw library
//!
//! This build script handles different platforms and library configurations:
//! - Windows MSVC: Uses bundled static libraries
//! - Windows GNU (MinGW): Uses bundled GNU static libraries or falls back to dynamic
//! - Linux/Mac: Tries system libraw first, then falls back to bundled GNU libraries
//!
//! The script automatically detects the target platform and configures linking accordingly.

use std::env;
use std::path::PathBuf;

fn main() {
    // 获取构建目标平台和项目根目录
    let target = env::var("TARGET").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    eprintln!("Building for target: {}", target);
    eprintln!("Project directory: {}", manifest_dir);

    // Windows MSVC 平台使用预编译的静态库
    if target.contains("msvc") {
        let lib_dir = PathBuf::from(&manifest_dir).join("libraw").join("msvc").join("lib");
        eprintln!("Using MSVC toolchain");
        eprintln!("Library directory: {}", lib_dir.display());

        // 配置 MSVC 静态库链接
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=libraw_static");

        // 告诉 cargo 在这些文件改变时重新运行构建脚本
        println!("cargo:rerun-if-changed=libraw/msvc/lib/libraw_static.lib");
        println!("cargo:rerun-if-changed=libraw/msvc/libraw/libraw.h");
    }
    // Windows GNU (MinGW) 平台
    else if target.contains("windows-gnu") {
        let lib_dir = PathBuf::from(&manifest_dir).join("libraw").join("gnu").join("lib");
        eprintln!("Using MinGW toolchain");
        eprintln!("Library directory: {}", lib_dir.display());

        // 设置库搜索路径
        println!("cargo:rustc-link-search=native={}", lib_dir.display());

        // 优先尝试静态链接，如果不存在则使用动态链接
        if lib_dir.join("libraw.a").exists() {
            eprintln!("Using static libraw library");
            println!("cargo:rustc-link-lib=static=raw");
            println!("cargo:rerun-if-changed=libraw/gnu/lib/libraw.a");
        } else {
            eprintln!("Static libraw not found, using dynamic library");
            println!("cargo:rustc-link-lib=dylib=raw");
        }

        // MinGW 需要链接 C++ 标准库
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rerun-if-changed=libraw/gnu/libraw/libraw.h");
    }
    // Linux/Mac 平台 - 优先使用系统库，如果没有则使用 bundled GNU 库
    else {
        // 1. 首先尝试使用 pkg-config 查找系统 libraw
        if std::process::Command::new("pkg-config").arg("--exists").arg("libraw").status().is_ok() {
            eprintln!("Using system libraw via pkg-config");
            println!("cargo:rustc-link-lib=dylib=raw");
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
        // 2. 检查常见的系统库路径
        else if std::path::Path::new("/usr/lib64/libraw.so").exists() {
            eprintln!("Using system libraw from /usr/lib64");
            println!("cargo:rustc-link-search=native=/usr/lib64");
            println!("cargo:rustc-link-lib=dylib=raw");
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
        // 3. 最后回退到 bundled GNU 库
        else {
            let lib_dir = PathBuf::from(&manifest_dir).join("libraw").join("gnu").join("lib");
            eprintln!("System libraw not found, using bundled GNU libraries");
            eprintln!("Library directory: {}", lib_dir.display());

            println!("cargo:rustc-link-search=native={}", lib_dir.display());

            // 优先尝试静态链接，如果不存在则使用动态链接
            if lib_dir.join("libraw.a").exists() {
                eprintln!("Using static libraw library from bundle");
                println!("cargo:rustc-link-lib=static=raw");
                println!("cargo:rerun-if-changed=libraw/gnu/lib/libraw.a");
            } else {
                eprintln!("Static library not found in bundle, expecting system dynamic library");
                println!("cargo:rustc-link-lib=dylib=raw");
            }

            // GNU 平台需要链接 C++ 标准库
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }

        // 监听头文件变化，确保在 API 更新时重新构建
        println!("cargo:rerun-if-changed=libraw/gnu/libraw/libraw.h");
    }

    // 监听构建脚本本身的变化
    println!("cargo:rerun-if-changed=build.rs");
}
