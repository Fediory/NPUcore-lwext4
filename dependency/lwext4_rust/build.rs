use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

fn main() {
    let c_path = PathBuf::from("c/lwext4")
        .canonicalize()
        .expect("cannot canonicalize path");

    let lwext4_make = Path::new("c/lwext4/toolchain/musl-generic.cmake");
    let lwext4_patch = Path::new("c/lwext4-make.patch").canonicalize().unwrap();

    if !Path::new(lwext4_make).exists() {
        println!("Retrieve lwext4 source code");
        let git_status = Command::new("git")
            .args(&["submodule", "update", "--init", "--recursive"])
            .status()
            .expect("failed to execute process: git submodule");
        assert!(git_status.success());

        println!("To patch lwext4 src");
        Command::new("git")
            .args(&["apply", lwext4_patch.to_str().unwrap()])
            .current_dir(c_path.clone())
            .spawn()
            .expect("failed to execute process: git apply patch");

        fs::copy(
            "c/musl-generic.cmake",
            "c/lwext4/toolchain/musl-generic.cmake",
        ).unwrap();
    }

    if !Path::new("c/lwext4/liblwext4-riscv64.a").exists() {
        let status = Command::new("make")
            .args(&[
                "musl-generic",
                "-C",
                c_path.to_str().expect("invalid path of lwext4"),
            ])
            .arg("ARCH=riscv64")
            .status()
            .expect("failed to execute process: make lwext4");
        assert!(status.success());

        let output = Command::new("riscv64-linux-musl-gcc")
            .args(["-print-sysroot"])
            .output()
            .expect("failed to execute process: gcc -print-sysroot");

        let sysroot = core::str::from_utf8(&output.stdout).unwrap();
        let sysroot = sysroot.trim_end();
        let sysroot_inc = &format!("-I{}/include/", sysroot);
        generates_bindings_to_rust(sysroot_inc);
    }

    /* No longer need to implement the libc.a
    let libc_name = &format!("c-{}", arch);
    let libc_dir = env::var("LIBC_BUILD_TARGET_DIR").unwrap_or(String::from("./"));
    let libc_dir = PathBuf::from(libc_dir)
        .canonicalize()
        .expect("cannot canonicalize LIBC_BUILD_TARGET_DIR");

    println!("cargo:rustc-link-lib=static={libc_name}");
    println!(
        "cargo:rustc-link-search=native={}",
        libc_dir.to_str().unwrap()
    );
    */

    println!("cargo:rustc-link-lib=static=lwext4-riscv64");
    println!(
        "cargo:rustc-link-search=native={}",
        c_path.to_str().unwrap()
    );
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=c/wrapper.h");
    println!("cargo:rerun-if-changed={}", c_path.to_str().unwrap());
}

fn generates_bindings_to_rust(mpath: &str) {
    let bindings = bindgen::Builder::default()
        .use_core()
        // The input header we would like to generate bindings for.
        .header("c/wrapper.h")
        .clang_arg(mpath)
        .clang_arg("-I./c/lwext4/include")
        .clang_arg("-I./c/lwext4/build_musl-generic/include/")
        .layout_tests(false)
        // Tell cargo to invalidate the built crate whenever any of the included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from("src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
