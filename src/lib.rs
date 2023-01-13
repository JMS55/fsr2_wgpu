mod fsr;

pub use fsr::{
    Fsr2FloatCoordinates, Fsr2InitializationFlags, Fsr2QualityMode, Fsr2Resolution, Fsr2Sharpen,
    Fsr2Texture,
};

use fsr::{
    ffxFsr2ContextCreate, ffxFsr2ContextDestroy, ffxFsr2ContextDispatch, FfxFsr2Context,
    FfxFsr2ContextDescription, FfxFsr2DispatchDescription, FfxFsr2Interface, FfxResource,
    FfxResourceStates_FFX_RESOURCE_STATE_COMPUTE_READ,
};
use fsr::{
    ffxFsr2GetInterfaceVK, ffxFsr2GetScratchMemorySizeVK, ffxGetDeviceVK, ffxGetTextureResourceVK,
};
use std::mem::MaybeUninit;
use std::ptr;
use std::time::Duration;
use wgpu::{CommandEncoder, Device};
use wgpu_core::api::Vulkan;

// TODO: Documentation for the whole library

pub struct Fsr2Context {
    context: FfxFsr2Context,
    upscaled_resolution: Fsr2Resolution,
    _scratch_memory: Vec<u8>,
}

impl Fsr2Context {
    pub fn new(
        device: &Device,
        max_input_resolution: Fsr2Resolution,
        upscaled_resolution: Fsr2Resolution,
        initialization_flags: Fsr2InitializationFlags,
    ) -> Self {
        unsafe {
            // Get underlying Vulkan objects from wgpu
            let (device, physical_device, get_device_proc_addr) =
                device.as_hal::<Vulkan, _, _>(|device| {
                    let device = device.unwrap();
                    let raw_device = device.raw_device().handle();
                    let physical_device = device.raw_physical_device();

                    let get_device_proc_addr = device
                        .shared_instance()
                        .raw_instance()
                        .fp_v1_0()
                        .get_device_proc_addr;

                    (raw_device, physical_device, get_device_proc_addr)
                });

            // Allocate scratch memory for FSR
            let scratch_memory_size = ffxFsr2GetScratchMemorySizeVK(physical_device);
            let mut scratch_memory = Vec::with_capacity(scratch_memory_size);

            // Setup an FSR->Vulkan interface
            let mut interface = MaybeUninit::<FfxFsr2Interface>::uninit();
            ffxFsr2GetInterfaceVK(
                interface.as_mut_ptr(),
                scratch_memory.as_mut_ptr() as *mut _,
                scratch_memory_size,
                physical_device,
                get_device_proc_addr,
            );
            let interface = interface.assume_init();

            // Create an FSR context
            let mut context = MaybeUninit::<FfxFsr2Context>::uninit();
            let context_description = FfxFsr2ContextDescription {
                flags: initialization_flags.bits(),
                maxRenderSize: max_input_resolution,
                displaySize: upscaled_resolution,
                callbacks: interface,
                device: ffxGetDeviceVK(device),
            };
            ffxFsr2ContextCreate(context.as_mut_ptr(), &context_description as *const _);
            let context = context.assume_init();

            Self {
                context,
                upscaled_resolution,
                _scratch_memory: scratch_memory,
            }
        }
    }

    pub fn set_new_upscale_resolution_if_changed(
        &mut self,
        new_upscaled_resolution: Fsr2Resolution,
    ) {
        if self.upscaled_resolution.width != new_upscaled_resolution.width
            || self.upscaled_resolution.height != new_upscaled_resolution.height
        {
            todo!("Recreate context, destroy old one");
        }
    }

    pub fn determine_input_resolution(&self, quality_mode: Fsr2QualityMode) -> Fsr2Resolution {
        let scale_factor = match quality_mode {
            Fsr2QualityMode::Quality => 1.5,
            Fsr2QualityMode::Balanced => 1.7,
            Fsr2QualityMode::Performance => 2.0,
            Fsr2QualityMode::UltraPerformance => 3.0,
        };

        Fsr2Resolution {
            width: (self.upscaled_resolution.width as f32 / scale_factor) as u32,
            height: (self.upscaled_resolution.height as f32 / scale_factor) as u32,
        }
    }

    pub fn render(
        &mut self,
        color: Fsr2Texture,
        depth: Fsr2Texture,
        motion_vectors: Fsr2Texture,
        motion_vector_scale: Option<Fsr2FloatCoordinates>,
        exposure: Option<Fsr2Texture>,
        pre_exposure: Option<f32>,
        // TODO: Change from Option to enum of disabled, user-provided, auto-generated
        reactive_mask: Option<Fsr2Texture>,
        transparency_and_composition_mask: Option<Fsr2Texture>,
        output: Fsr2Texture,
        input_resolution: Fsr2Resolution,
        sharpen: Fsr2Sharpen,
        frame_delta_time: Duration,
        reset: bool,
        camera_near: f32,
        camera_far: Option<f32>,
        camera_fov_angle_vertical: f32,
        jitter_offset: Fsr2FloatCoordinates,
        command_encoder: &CommandEncoder,
    ) {
        unsafe {
            let dispatch_description = FfxFsr2DispatchDescription {
                commandList: todo!(),
                color: self.texture_to_ffx_resource(Some(color)),
                depth: self.texture_to_ffx_resource(Some(depth)),
                motionVectors: self.texture_to_ffx_resource(Some(motion_vectors)),
                exposure: self.texture_to_ffx_resource(exposure),
                reactive: self.texture_to_ffx_resource(reactive_mask),
                transparencyAndComposition: self
                    .texture_to_ffx_resource(transparency_and_composition_mask),
                output: self.texture_to_ffx_resource(Some(output)),
                jitterOffset: jitter_offset,
                motionVectorScale: motion_vector_scale
                    .unwrap_or(Fsr2FloatCoordinates { x: 1.0, y: 1.0 }),
                renderSize: input_resolution,
                enableSharpening: !matches!(sharpen, Fsr2Sharpen::Disabled),
                sharpness: match sharpen {
                    Fsr2Sharpen::Disabled => 0.0,
                    Fsr2Sharpen::Enabled(sharpness) => sharpness.clamp(0.0, 1.0),
                },
                frameTimeDelta: frame_delta_time.as_millis() as f32,
                preExposure: pre_exposure.unwrap_or(1.0),
                reset,
                cameraNear: camera_near,
                cameraFar: camera_far.unwrap_or(0.0),
                cameraFovAngleVertical: camera_fov_angle_vertical,
            };

            ffxFsr2ContextDispatch(
                &mut self.context as *mut _,
                &dispatch_description as *const _,
            );
        }
    }

    unsafe fn texture_to_ffx_resource(&mut self, texture: Option<Fsr2Texture>) -> FfxResource {
        match texture {
            Some(texture) => todo!(),
            // Some(texture) => ffxGetTextureResourceVK(
            //     &mut self.context as *mut _,
            //     texture
            //         .texture
            //         .as_hal::<Vulkan, _>(|x| x.unwrap().raw_handle()),
            //     texture
            //         .view
            //         .as_hal::<Vulkan, _>(|t| x.unwrap().raw_handle()),
            //     texture.width,
            //     texture.height,
            //     texture.format,
            //     ptr::null_mut(),
            //     FfxResourceStates_FFX_RESOURCE_STATE_COMPUTE_READ,
            // ),
            None => todo!(),
        }
    }
}

impl Drop for Fsr2Context {
    fn drop(&mut self) {
        unsafe {
            // TODO: Wait for FSR resources to not be in use somehow
            ffxFsr2ContextDestroy(&mut self.context as *mut _);
        }
    }
}
