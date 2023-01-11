use bindgen::Builder;
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=native=./fsr2");
    println!("cargo:rustc-link-search=native=./fsr2/vk");
    println!("cargo:rustc-link-lib=static=ffx_fsr2_api_x86_64");
    println!("cargo:rustc-link-lib=static=ffx_fsr2_api_vk_x86_64");

    let bindings = Builder::default()
        .header("fsr2/ffx_fsr2.h")
        .header("fsr2/vk/ffx_fsr2_vk.h")
        .clang_args(["-x", "c++", "-fdeclspec"])
        .allowlist_type("FfxFsr2.*")
        .allowlist_function("ffxFsr2.*")
        .blocklist_type("Vk.*")
        .blocklist_type(".*vk.*")
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}
