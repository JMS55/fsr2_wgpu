mod fsr;

pub use fsr::{
    Fsr2Exposure, Fsr2InitializationFlags, Fsr2QualityMode, Fsr2ReactiveMask, Fsr2Texture,
};

use fsr::{
    ffxFsr2ContextCreate, ffxFsr2ContextDestroy, ffxFsr2ContextDispatch, ffxFsr2GetJitterOffset,
    ffxFsr2GetJitterPhaseCount, FfxDimensions2D, FfxFloatCoords2D, FfxFsr2Context,
    FfxFsr2ContextDescription, FfxFsr2DispatchDescription, FfxFsr2Interface, FfxResource,
    FfxResourceStates_FFX_RESOURCE_STATE_COMPUTE_READ,
};
use fsr::{
    ffxFsr2GetInterfaceVK, ffxFsr2GetScratchMemorySizeVK, ffxGetCommandListVK, ffxGetDeviceVK,
    ffxGetTextureResourceVK,
};
use glam::{Mat4, UVec2, Vec2, Vec3};
use std::mem::MaybeUninit;
use std::ptr;
use std::time::Duration;
use wgpu::{Adapter, CommandEncoder, Device};
use wgpu_core::api::Vulkan;

// TODO: Documentation for the whole library

// TODO: Thread safety?
pub struct Fsr2Context {
    context: FfxFsr2Context,
    upscaled_resolution: UVec2,
    _scratch_memory: Vec<u8>,
}

impl Fsr2Context {
    pub fn new(
        device: &Device,
        max_input_resolution: UVec2,
        upscaled_resolution: UVec2,
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
                maxRenderSize: uvec2_to_dim2d(max_input_resolution),
                displaySize: uvec2_to_dim2d(upscaled_resolution),
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

    pub fn set_new_upscale_resolution_if_changed(&mut self, new_upscaled_resolution: UVec2) {
        if new_upscaled_resolution != self.upscaled_resolution {
            todo!("Recreate context, destroy old one");
        }
    }

    pub fn get_suggested_input_resolution(&self, quality_mode: Fsr2QualityMode) -> UVec2 {
        let scale_factor = match quality_mode {
            Fsr2QualityMode::Quality => 1.5,
            Fsr2QualityMode::Balanced => 1.7,
            Fsr2QualityMode::Performance => 2.0,
            Fsr2QualityMode::UltraPerformance => 3.0,
        };

        (self.upscaled_resolution.as_vec2() / scale_factor).as_uvec2()
    }

    pub fn jitter_camera_projection_matrix(
        &self,
        matrix: &mut Mat4,
        input_resolution: UVec2,
        frame_index: i32,
    ) -> Vec2 {
        let jitter_offset = self.get_camera_jitter_offset(input_resolution, frame_index);

        let mut translation = 2.0 * jitter_offset / input_resolution.as_vec2();
        translation.y *= -1.0;

        let translation = Mat4::from_translation(Vec3 {
            x: translation.x,
            y: translation.y,
            z: 0.0,
        });
        *matrix = translation * *matrix;

        jitter_offset
    }

    pub fn get_camera_jitter_offset(&self, input_resolution: UVec2, frame_index: i32) -> Vec2 {
        unsafe {
            let phase_count = ffxFsr2GetJitterPhaseCount(
                input_resolution.x.try_into().unwrap(),
                self.upscaled_resolution.x.try_into().unwrap(),
            );

            let mut jitter_offset = Vec2::ZERO;
            ffxFsr2GetJitterOffset(
                &mut jitter_offset.x as *mut _,
                &mut jitter_offset.y as *mut _,
                frame_index,
                phase_count,
            );
            jitter_offset
        }
    }

    pub fn get_mip_bias(&self, input_resolution: UVec2) -> f32 {
        (input_resolution.x as f32 / self.upscaled_resolution.x as f32).log2() - 1.0
    }

    pub fn render(
        &mut self,
        color: Fsr2Texture,
        depth: Fsr2Texture,
        motion_vectors: Fsr2Texture,
        motion_vector_scale: Option<Vec2>,
        exposure: Fsr2Exposure,
        reactive_mask: Fsr2ReactiveMask,
        transparency_and_composition_mask: Option<Fsr2Texture>,
        output: Fsr2Texture,
        input_resolution: UVec2,
        sharpness: f32,
        frame_delta_time: Duration,
        reset: bool,
        camera_near: f32,
        camera_far: Option<f32>,
        camera_fov_angle_vertical: f32,
        jitter_offset: Vec2,
        adapter: &Adapter,
        command_encoder: &mut CommandEncoder,
    ) {
        unsafe {
            let (exposure, pre_exposure) = match exposure {
                Fsr2Exposure::AutoExposure => (None, 0.0),
                Fsr2Exposure::ManualExposure {
                    pre_exposure,
                    exposure,
                } => (Some(exposure), pre_exposure),
            };

            let reactive = match reactive_mask {
                Fsr2ReactiveMask::NoMask => self.texture_to_ffx_resource(None, adapter),
                Fsr2ReactiveMask::ManualMask(mask) => {
                    self.texture_to_ffx_resource(Some(mask), adapter)
                }
                Fsr2ReactiveMask::AutoMask {
                    color_opaque_only,
                    color_opaque_and_transparent,
                    scale,
                    threshold,
                    binary_value,
                    flags,
                } => todo!(),
            };

            let dispatch_description = FfxFsr2DispatchDescription {
                commandList: ffxGetCommandListVK(
                    command_encoder.as_hal_mut::<Vulkan, _, _>(|x| x.unwrap().raw_handle()),
                ),
                color: self.texture_to_ffx_resource(Some(color), adapter),
                depth: self.texture_to_ffx_resource(Some(depth), adapter),
                motionVectors: self.texture_to_ffx_resource(Some(motion_vectors), adapter),
                exposure: self.texture_to_ffx_resource(exposure, adapter),
                reactive,
                transparencyAndComposition: self
                    .texture_to_ffx_resource(transparency_and_composition_mask, adapter),
                output: self.texture_to_ffx_resource(Some(output), adapter),
                jitterOffset: vec2_to_float_coords2d(jitter_offset),
                motionVectorScale: vec2_to_float_coords2d(motion_vector_scale.unwrap_or(Vec2::ONE)),
                renderSize: uvec2_to_dim2d(input_resolution),
                enableSharpening: sharpness > 0.0,
                sharpness: sharpness.clamp(0.0, 1.0),
                frameTimeDelta: frame_delta_time.as_millis() as f32,
                preExposure: pre_exposure,
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

    unsafe fn texture_to_ffx_resource(
        &mut self,
        texture: Option<Fsr2Texture>,
        adapter: &Adapter,
    ) -> FfxResource {
        if let Some(Fsr2Texture { texture, view }) = texture {
            ffxGetTextureResourceVK(
                &mut self.context as *mut _,
                texture.as_hal::<Vulkan, _, _>(|x| x.unwrap().raw_handle()),
                view.as_hal::<Vulkan, _, _>(|x| x.unwrap().raw_handle()),
                texture.width(),
                texture.height(),
                adapter
                    .texture_format_as_hal::<Vulkan>(texture.format())
                    .unwrap(),
                ptr::null_mut(),
                FfxResourceStates_FFX_RESOURCE_STATE_COMPUTE_READ,
            )
        } else {
            ffxGetTextureResourceVK(
                &mut self.context as *mut _,
                ash::vk::Image::null(),
                ash::vk::ImageView::null(),
                1,
                1,
                ash::vk::Format::UNDEFINED,
                ptr::null_mut(),
                FfxResourceStates_FFX_RESOURCE_STATE_COMPUTE_READ,
            )
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

fn uvec2_to_dim2d(vec: UVec2) -> FfxDimensions2D {
    FfxDimensions2D {
        width: vec.x,
        height: vec.y,
    }
}

fn vec2_to_float_coords2d(vec: Vec2) -> FfxFloatCoords2D {
    FfxFloatCoords2D { x: vec.x, y: vec.y }
}
