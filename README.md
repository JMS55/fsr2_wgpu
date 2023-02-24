# fsr2_wgpu
### FidelityFX Super Resolution 2 for wgpu

## Building FSR2 Static Libraries
### Windows
* Install Visual Studio, Clang, CMake, and Git
* Clone https://github.com/GPUOpen-Effects/FidelityFX-FSR2
* Run `FidelityFX-FSR2\GenerateSolutions.bat`
* Open `FidelityFX-FSR2\build\VK\FSR2_Sample_VK.sln`
* Edit the `Debug` configurations of `ffx_fsr2_api_x64` and `ffx_fsr2_api_vk_x64` to compile with `/MD` instead of the default `/MDd`
    * This is a workaround for the following [rustc issue](https://github.com/rust-lang/rust/issues/39016)
* Build `ffx_fsr2_api_x64` and `ffx_fsr2_api_vk_x64` in both `Debug` and `Release` configurations
* Copy the 4 static libraries from `FidelityFX-FSR2\bin\ffx_fsr2_api` to `fsr2_wgpu\fsr2\lib`

## Linux
* TODO
