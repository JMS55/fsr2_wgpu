use bindgen::Builder;
use std::env;
use std::path::PathBuf;

fn main() {
    let vulkan_sdk = env::var("VULKAN_SDK").expect("VULKAN_SDK environment variable not set");

    println!("cargo:rustc-link-search=native=./fsr2/lib");
    #[cfg(debug_assertions)]
    {
        println!("cargo:rustc-link-lib=static=ffx_fsr2_api_x86_64d");
        println!("cargo:rustc-link-lib=static=ffx_fsr2_api_vk_x86_64d");
    }
    #[cfg(not(debug_assertions))]
    {
        println!("cargo:rustc-link-lib=static=ffx_fsr2_api_x86_64");
        println!("cargo:rustc-link-lib=static=ffx_fsr2_api_vk_x86_64");
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!("cargo:rustc-link-search=native={vulkan_sdk}/lib");
        println!("cargo:rustc-link-lib=dylib=vulkan");
    }
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-search=native={vulkan_sdk}/Lib");
        println!("cargo:rustc-link-lib=dylib=vulkan-1");
    }

    #[cfg(not(target_os = "windows"))]
    let vulkan_sdk_include = "include";
    #[cfg(target_os = "windows")]
    let vulkan_sdk_include = "Include";

    let bindings = Builder::default()
        .header("fsr2/include/ffx_fsr2.h")
        .header("fsr2/include/vk/ffx_fsr2_vk.h")
        .clang_args(["-x", "c++"])
        .clang_arg("-fdeclspec")
        .clang_arg(format!("-I{vulkan_sdk}/{vulkan_sdk_include}"))
        .clang_arg("-stdlib=libc++")
        .blocklist_type("VkPhysicalDevice")
        .blocklist_type("VkDevice")
        .blocklist_type("VkImage")
        .blocklist_type("VkImageView")
        .blocklist_type("VkFormat")
        .blocklist_type("VkCommandBuffer")
        .blocklist_type("PFN_vkGetDeviceProcAddr")
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}
