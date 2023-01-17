use arrayvec::ArrayVec;
use ash::vk::{
    AccessFlags, CommandBuffer, DependencyFlags, Image, ImageLayout, ImageMemoryBarrier,
    PipelineStageFlags, StructureType, QUEUE_FAMILY_IGNORED,
};
use std::ptr;
use wgpu::Device;
use wgpu_core::api::Vulkan;

#[derive(Default)]
pub struct Barriers {
    current_pipeline_stage: PipelineStageFlags,
    start_barriers: ArrayVec<ImageMemoryBarrier, 7>,
    end_barriers: ArrayVec<ImageMemoryBarrier, 7>,
}

impl Barriers {
    pub fn add(&mut self, image: Image) {
        let current_access_mask = todo!();
        let current_layout = todo!();
        let subresource_range = todo!();
        self.current_pipeline_stage |= todo!();

        self.start_barriers.push(ImageMemoryBarrier {
            s_type: StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: current_access_mask,
            dst_access_mask: AccessFlags::SHADER_READ,
            old_layout: current_layout,
            new_layout: ImageLayout::READ_ONLY_OPTIMAL,
            src_queue_family_index: QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: QUEUE_FAMILY_IGNORED,
            image,
            subresource_range,
        });

        self.end_barriers.push(ImageMemoryBarrier {
            s_type: StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            dst_access_mask: current_access_mask,
            src_access_mask: AccessFlags::SHADER_READ,
            old_layout: ImageLayout::READ_ONLY_OPTIMAL,
            new_layout: current_layout,
            src_queue_family_index: QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: QUEUE_FAMILY_IGNORED,
            image,
            subresource_range,
        });
    }

    // TODO: Get Device from CommandEncoder instead

    pub unsafe fn cmd_start(&self, command_buffer: CommandBuffer, device: &Device) {
        device.as_hal::<Vulkan, _, _>(|device| {
            device.unwrap().raw_device().cmd_pipeline_barrier(
                command_buffer,
                self.current_pipeline_stage,
                PipelineStageFlags::COMPUTE_SHADER,
                DependencyFlags::empty(),
                &[],
                &[],
                &self.start_barriers,
            )
        });
    }

    pub unsafe fn cmd_end(&self, command_buffer: CommandBuffer, device: &Device) {
        device.as_hal::<Vulkan, _, _>(|device| {
            device.unwrap().raw_device().cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::COMPUTE_SHADER,
                self.current_pipeline_stage,
                DependencyFlags::empty(),
                &[],
                &[],
                &self.end_barriers,
            )
        });
    }
}
