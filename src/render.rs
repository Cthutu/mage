//
// ASCII renderer
//

#![allow(unused_variables)]

use thiserror::Error;
use wgpu::{
    Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, PresentMode, Queue,
    RequestAdapterOptions, RequestDeviceError, Surface, SwapChain, SwapChainDescriptor,
    SwapChainError, TextureUsage,
};
use winit::{dpi::PhysicalSize, window::Window};

//
// Rendering system errors that are passed into Results
//

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Appropriate graphics device was not found")]
    BadAdapter,

    #[error(transparent)]
    BadDevice(#[from] RequestDeviceError),

    #[error("Could not find a texture format compatible with the swap chain")]
    BadSwapChainFormat,
}

pub type RenderResult<T> = Result<T, RenderError>;

//
// Rendering state and interface
//

pub struct RenderState {
    surface: Surface,
    device: Device,
    queue: Queue,
    swapchain_desc: SwapChainDescriptor,
    swapchain: SwapChain,
}

impl RenderState {
    pub async fn new(window: &Window) -> RenderResult<Self> {
        let inner_size = window.inner_size();

        // An instance represents access to the WGPU API.  Here we decide which
        // back-end to use (Vulkan, DX12, Metal etc), but we let WGPU decide by
        // stating PRIMARY.
        let instance = Instance::new(wgpu::BackendBit::PRIMARY);

        // This can be unsafe since we know the window has a valid window
        // handle, otherwise we wouldn't get here.  The surface is an interface
        // to the OS window that will host the rendering.
        let surface = unsafe { instance.create_surface(window) };

        // The adapter represents a physical graphics/compute device.  We need a
        // device that can handle the surface we will be rendering to.
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or(RenderError::BadAdapter)?;

        // Now we create the device and queue from the adapter.  A device is a
        // logical software construct around the physical device.  It serves as
        // the interface for creating many resources.  A queue is used to
        // deliver commands to the GPU to carry out actions, such as writing to
        // texture buffers.
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Render device"),
                    features: Features::empty(),
                    limits: Limits::default(),
                },
                None,
            )
            .await?;

        // We create the swap chain descriptor that provides the configuration
        // for creating the swap chain.  However, we keep it around because we
        // need to recreate the swap chain every time the window resizes.
        let swapchain_desc = SwapChainDescriptor {
            usage: TextureUsage::RENDER_ATTACHMENT,
            format: adapter
                .get_swap_chain_preferred_format(&surface)
                .ok_or(RenderError::BadSwapChainFormat)?,
            width: inner_size.width,
            height: inner_size.height,
            present_mode: PresentMode::Fifo,
        };

        // Now we create the swap chain that will target a particular surface.
        let swapchain = device.create_swap_chain(&surface, &swapchain_desc);

        Ok(RenderState {
            surface,
            device,
            queue,
            swapchain_desc,
            swapchain,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.swapchain_desc.width = new_size.width;
        self.swapchain_desc.height = new_size.height;
        self.swapchain = self
            .device
            .create_swap_chain(&self.surface, &self.swapchain_desc);
    }

    pub fn render(&mut self) -> Result<(), SwapChainError> {
        todo!()
    }
}
