use arrayvec::ArrayVec;
use ash::vk::{
    AccessFlags, CommandBuffer, DependencyFlags, ImageLayout, ImageMemoryBarrier,
    ImageSubresourceRange, PipelineStageFlags, StructureType, QUEUE_FAMILY_IGNORED,
};
use std::ptr;
use wgpu::{Device, Texture};
use wgpu_core::api::Vulkan;
use wgpu_hal::vulkan::conv;

#[derive(Default)]
pub struct Barriers {
    current_pipeline_stage: PipelineStageFlags,
    start_barriers: ArrayVec<ImageMemoryBarrier, 7>,
    end_barriers: ArrayVec<ImageMemoryBarrier, 7>,
}

impl Barriers {
    pub unsafe fn add(&mut self, texture: &Texture, new_layout: ImageLayout) {
        let (image, usage, aspects) = texture.as_hal::<Vulkan, _, _>(|texture| {
            let texture = texture.unwrap();
            (texture.raw_handle(), texture.usage, texture.aspects)
        });

        let (current_pipeline_stage, current_access_mask) =
            conv::map_texture_usage_to_barrier(usage);
        let current_layout = conv::derive_image_layout(usage, aspects);
        let subresource_range = ImageSubresourceRange {
            aspect_mask: conv::map_aspects(aspects),
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };

        self.current_pipeline_stage |= current_pipeline_stage;

        self.start_barriers.push(ImageMemoryBarrier {
            s_type: StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: current_access_mask,
            dst_access_mask: AccessFlags::SHADER_READ,
            old_layout: current_layout,
            new_layout,
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
            old_layout: new_layout,
            new_layout: current_layout,
            src_queue_family_index: QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: QUEUE_FAMILY_IGNORED,
            image,
            subresource_range,
        });
    }

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
