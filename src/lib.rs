mod fsr;

pub use fsr::{FfxDimensions2D, Fsr2InitializationFlags};

use fsr::{
    ffxFsr2ContextCreate, ffxFsr2ContextDestroy, ffxFsr2ContextDispatch, FfxFsr2Context,
    FfxFsr2ContextDescription, FfxFsr2DispatchDescription, FfxFsr2Interface,
};
use fsr::{ffxFsr2GetInterfaceVK, ffxFsr2GetScratchMemorySizeVK};
use std::mem::MaybeUninit;
use wgpu::Device;
use wgpu_core::api::Vulkan;

pub struct Fsr2Context {
    context: FfxFsr2Context,
    scratch_memory: Vec<u8>, // TODO: Hold Box<[u8]> instead
}

impl Fsr2Context {
    pub fn new(
        device: &Device,
        max_display_size: FfxDimensions2D,
        upscale_render_size: FfxDimensions2D,
        initialization_flags: Fsr2InitializationFlags,
    ) -> Self {
        unsafe {
            // Get underlying Vulkan objects
            let (mut device, physical_device, get_device_proc_addr) = device
                .as_hal::<Vulkan, _, _>(|device| {
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
            let interface = interface.assume_init(); // TODO: Need to store interface in rust_context?

            // Create an FSR context
            let mut context = MaybeUninit::<FfxFsr2Context>::uninit();
            let context_description = FfxFsr2ContextDescription {
                flags: initialization_flags.bits(),
                maxRenderSize: max_display_size,
                displaySize: upscale_render_size,
                callbacks: interface,
                device: &mut device as *mut _ as *mut _,
            };
            ffxFsr2ContextCreate(context.as_mut_ptr(), &context_description as *const _);
            let context = context.assume_init();

            Self {
                context,
                scratch_memory,
            }
        }
    }

    pub fn render(&mut self) {
        let dispatch_description = FfxFsr2DispatchDescription {
            commandList: todo!(),
            color: todo!(),
            depth: todo!(),
            motionVectors: todo!(),
            exposure: todo!(),
            reactive: todo!(),
            transparencyAndComposition: todo!(),
            output: todo!(),
            jitterOffset: todo!(),
            motionVectorScale: todo!(),
            renderSize: todo!(),
            enableSharpening: todo!(),
            sharpness: todo!(),
            frameTimeDelta: todo!(),
            preExposure: todo!(),
            reset: todo!(),
            cameraNear: todo!(),
            cameraFar: todo!(),
            cameraFovAngleVertical: todo!(),
        };

        unsafe {
            ffxFsr2ContextDispatch(
                &mut self.context as *mut _,
                &dispatch_description as *const _,
            );
        }
    }
}

impl Drop for Fsr2Context {
    fn drop(&mut self) {
        unsafe {
            // TODO: Wait for FSR resources to not be in use
            ffxFsr2ContextDestroy(&mut self.context as *mut _);
        }
    }
}
