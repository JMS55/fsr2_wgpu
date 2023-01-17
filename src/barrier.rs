use arrayvec::ArrayVec;
use ash::vk::{CommandBuffer, DependencyFlags, Image, ImageMemoryBarrier, PipelineStageFlags};
use wgpu::Device;
use wgpu_core::api::Vulkan;

#[derive(Default)]
pub struct Barriers {
    start_pipeline_stage: PipelineStageFlags,
    start_barriers: ArrayVec<ImageMemoryBarrier, 7>,
    end_barriers: ArrayVec<ImageMemoryBarrier, 7>,
}

impl Barriers {
    pub fn add(&mut self, image: Image) {
        todo!()
    }

    // TODO: Get Device from CommandEncoder instead

    pub fn cmd_start(&mut self, command_buffer: CommandBuffer, device: &Device) {
        unsafe {
            device.as_hal::<Vulkan, _, _>(|device| {
                device.unwrap().raw_device().cmd_pipeline_barrier(
                    command_buffer,
                    self.start_pipeline_stage,
                    PipelineStageFlags::COMPUTE_SHADER,
                    DependencyFlags::empty(),
                    &[],
                    &[],
                    &self.start_barriers,
                )
            });
        }
    }

    pub fn cmd_end(&mut self, command_buffer: CommandBuffer, device: &Device) {
        unsafe {
            device.as_hal::<Vulkan, _, _>(|device| {
                device.unwrap().raw_device().cmd_pipeline_barrier(
                    command_buffer,
                    PipelineStageFlags::COMPUTE_SHADER,
                    self.start_pipeline_stage,
                    DependencyFlags::empty(),
                    &[],
                    &[],
                    &self.end_barriers,
                )
            });
        }
    }
}
