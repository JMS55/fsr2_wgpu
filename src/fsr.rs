#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#![allow(improper_ctypes)] // https://github.com/rust-lang/rust-bindgen/issues/1549

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

type VkPhysicalDevice = ash::vk::PhysicalDevice;
type VkDevice = ash::vk::Device;
type VkImage = ash::vk::Image;
type VkImageView = ash::vk::ImageView;
type VkFormat = ash::vk::Format;
type VkCommandBuffer = ash::vk::CommandBuffer;
type PFN_vkGetDeviceProcAddr = ash::vk::PFN_vkGetDeviceProcAddr;

bitflags::bitflags! {
    pub struct Fsr2InitializationFlags: FfxFsr2InitializationFlagBits {
        const AUTO_EXPOSURE = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_AUTO_EXPOSURE;
        const INFINITE_DEPTH = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_DEPTH_INFINITE;
        const INVERTED_DEPTH = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_DEPTH_INVERTED;
        const DISPLAY_RESOLUTION_MOTION_VECTORS = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS;
        const JITTER_CANCELLED_MOTION_VECTORS = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION;
        const DYNAMIC_RESOLUTION = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_DYNAMIC_RESOLUTION;
        const HIGH_DYNAMIC_RANGE = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_HIGH_DYNAMIC_RANGE;
        const TEXTURE_1D = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_TEXTURE1D_USAGE;
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Fsr2WgpuError {
    #[error(transparent)]
    Fsr2(#[from] Fsr2Error),
    #[error(transparent)]
    Wgpu(#[from] wgpu_hal::DeviceError),
    #[error(transparent)]
    Vulkan(#[from] ash::vk::Result),
}

#[derive(thiserror::Error, Debug)]
pub enum Fsr2Error {
    #[error("TODO")]
    InvalidPointer,
    #[error("TODO")]
    InvalidAlignment,
    #[error("TODO")]
    InvalidSize,
    #[error("TODO")]
    Eof,
    #[error("TODO")]
    InvalidPath,
    #[error("TODO")]
    ErrorEof,
    #[error("TODO")]
    MalformedData,
    #[error("TODO")]
    OutOfMemory,
    #[error("TODO")]
    IncompleteInterface,
    #[error("TODO")]
    InvalidEnum,
    #[error("TODO")]
    InvalidArgument,
    #[error("TODO")]
    OutOfRange,
    #[error("TODO")]
    NullDevice,
    #[error("TODO")]
    BackendApiError,
    #[error("TODO")]
    InsufficentMemory,
}

pub fn ffx_check_result(result: FfxErrorCode) -> Result<(), Fsr2Error> {
    match result {
        FFX_OK => Ok(()),
        FFX_ERROR_INVALID_POINTER => Err(Fsr2Error::InvalidPointer),
        FFX_ERROR_INVALID_ALIGNMENT => Err(Fsr2Error::InvalidAlignment),
        FFX_ERROR_INVALID_SIZE => Err(Fsr2Error::InvalidSize),
        FFX_EOF => Err(Fsr2Error::Eof),
        FFX_ERROR_INVALID_PATH => Err(Fsr2Error::InvalidPath),
        FFX_ERROR_EOF => Err(Fsr2Error::ErrorEof),
        FFX_ERROR_MALFORMED_DATA => Err(Fsr2Error::MalformedData),
        FFX_ERROR_OUT_OF_MEMORY => Err(Fsr2Error::OutOfMemory),
        FFX_ERROR_INCOMPLETE_INTERFACE => Err(Fsr2Error::IncompleteInterface),
        FFX_ERROR_INVALID_ENUM => Err(Fsr2Error::InvalidEnum),
        FFX_ERROR_INVALID_ARGUMENT => Err(Fsr2Error::InvalidArgument),
        FFX_ERROR_OUT_OF_RANGE => Err(Fsr2Error::OutOfRange),
        FFX_ERROR_NULL_DEVICE => Err(Fsr2Error::NullDevice),
        FFX_ERROR_BACKEND_API_ERROR => Err(Fsr2Error::BackendApiError),
        FFX_ERROR_INSUFFICIENT_MEMORY => Err(Fsr2Error::InsufficentMemory),
        _ => unreachable!(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fsr2QualityMode {
    Quality,
    Balanced,
    Performance,
    UltraPerformance,
}

pub struct Fsr2Texture<'a> {
    pub texture: &'a wgpu::Texture,
    pub view: &'a wgpu::TextureView,
}

pub enum Fsr2Exposure<'a> {
    AutoExposure,
    ManualExposure {
        pre_exposure: f32,
        exposure: Fsr2Texture<'a>,
    },
}

pub enum Fsr2ReactiveMask<'a> {
    NoMask,
    ManualMask(Fsr2Texture<'a>),
    AutoMask {
        color_opaque_only: Fsr2Texture<'a>,
        color_opaque_and_transparent: Fsr2Texture<'a>,
        scale: f32,
        threshold: f32,
        binary_value: f32,
        flags: Fsr2AutoGenerateReactiveMaskFlags,
    },
}

bitflags::bitflags! {
    pub struct Fsr2AutoGenerateReactiveMaskFlags: u32 {
      const ApplyTonemap = FFX_FSR2_AUTOREACTIVEFLAGS_APPLY_TONEMAP;
      const ApplyInverseTonemap = FFX_FSR2_AUTOREACTIVEFLAGS_APPLY_INVERSETONEMAP;
      const ApplyThreshold = FFX_FSR2_AUTOREACTIVEFLAGS_APPLY_THRESHOLD;
      const UseComponentsMax = FFX_FSR2_AUTOREACTIVEFLAGS_USE_COMPONENTS_MAX;
    }
}
