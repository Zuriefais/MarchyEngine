use crate::egui_tools::EguiRenderer;
use crate::gui::EngineGui;

use crate::render_passes::render_pass_manager::RenderPassManager;

use crate::styles::default_dark::default_dark_theme;
use egui_wgpu::wgpu::SurfaceError;
use egui_wgpu::{ScreenDescriptor, wgpu};
use log::info;
use std::sync::Arc;
use wgpu::{ExperimentalFeatures, Instance, Limits, PresentMode};

use winit::event::WindowEvent;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
use winit::window::Window;

pub struct AppState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,
    pub engine_gui: EngineGui,
    pub window: Arc<Window>,
    render_pass_manager: RenderPassManager,
    present_mode: PresentMode,
    vsync_enabled: bool,
    instance: Instance,
    recreate_render_pass_manager: bool,
}

impl AppState {
    pub async fn new(window: Arc<Window>, _cli_module: Option<String>) -> anyhow::Result<Self> {
        info!("Creating App State...");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::all(),
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");
        info!("Surface created");

        // Get actual window size (important for Wayland)
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        info!("Initial window size: {}x{}", width, height);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Unable request adapter");
        info!(
            "Adapter features: {:#}, limits: {:#?}",
            adapter.features(),
            adapter.limits()
        );

        let features = wgpu::Features::PUSH_CONSTANTS
            | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
            | wgpu::Features::FLOAT32_FILTERABLE;
        let mut limits = Limits::default();
        limits.max_push_constant_size = 128;
        limits.max_compute_workgroup_size_x = 64;
        limits.max_compute_workgroup_size_y = 64;
        limits.max_compute_invocations_per_workgroup = 1024;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: features,
                required_limits: limits,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
                experimental_features: ExperimentalFeatures::disabled(),
            })
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        info!("Supported formats: {:?}", swapchain_capabilities.formats);
        #[cfg(target_os = "android")]
        let selected_format = wgpu::TextureFormat::Rgba32Float;
        #[cfg(not(target_os = "android"))]
        let selected_format = wgpu::TextureFormat::Bgra8Unorm;
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("failed to select proper surface texture format!");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let egui_renderer = EguiRenderer::new(&device, surface_config.format, None, 1, &window);

        // Set default egui font
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Adwaita Sans".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "styles/AdwaitaSans-Regular.ttf"
            ))),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "Adwaita Sans".to_owned());
        egui_renderer.context().set_fonts(fonts);

        // Set egui style
        egui_renderer.context().set_style(default_dark_theme());

        let scale_factor = 1.0;

        let engine_gui = EngineGui::new(egui_renderer.context());

        let render_pass_manager =
            RenderPassManager::new(&device, &queue, &surface_config, width, height);

        info!("App State created!!");

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,
            scale_factor,
            engine_gui,
            window,
            render_pass_manager,

            present_mode: wgpu::PresentMode::AutoVsync,
            vsync_enabled: true,
            instance,
            recreate_render_pass_manager: false,
        })
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        puffin::profile_function!();

        // Skip invalid sizes
        if width == 0 || height == 0 {
            log::warn!("Skipping resize to invalid size: {}x{}", width, height);
            return;
        }

        // Skip if size hasn't changed
        if self.surface_config.width == width && self.surface_config.height == height {
            return;
        }

        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);

        // Resize render passes immediately (deferred resize caused Wayland issues)
        self.render_pass_manager
            .resize(width, height, &self.device, &self.queue);
    }

    pub fn set_vsync_enabled(&mut self, enabled: bool) {
        let new_present_mode = if enabled {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };

        if self.present_mode != new_present_mode {
            self.present_mode = new_present_mode;
            self.surface_config.present_mode = new_present_mode;

            self.surface.configure(&self.device, &self.surface_config);
            log::info!("V-sync changed to: {:?}", new_present_mode);
        }
    }

    pub fn handle_redraw(&mut self) {
        puffin::profile_function!();
        puffin::GlobalProfiler::lock().new_frame();

        let width = self.surface_config.width;
        let height = self.surface_config.height;

        // Skip rendering if size is invalid
        if width == 0 || height == 0 {
            self.window.request_redraw();
            return;
        }

        if self.recreate_render_pass_manager {
            self.recreate_render_pass_manager = false;
            let options = self.render_pass_manager.get_options().clone();
            self.render_pass_manager = RenderPassManager::new(
                &self.device,
                &self.queue,
                &self.surface_config,
                width,
                height,
            );
            *self.render_pass_manager.get_options() = options;
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: self.window.scale_factor() as f32 * self.scale_factor,
        };

        let surface_texture = match self.surface.get_current_texture() {
            Ok(st) => st,
            Err(SurfaceError::Outdated) | Err(SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.surface_config);
                return;
            }
            Err(SurfaceError::OutOfMemory) => {
                panic!("Surface out of memory");
            }
            Err(e) => {
                log::warn!("Surface error: {:?}", e);
                return;
            }
        };
        let mut need_reconfigure = false;

        {
            let surface_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let vsync_enabled = self.vsync_enabled;
            self.render_pass_manager
                .render(&self.queue, &surface_view, &mut encoder, &self.device);

            // Begin egui frame before rendering GUI
            self.egui_renderer.begin_frame(&self.window);

            self.engine_gui.render_gui(
                self.render_pass_manager.get_options(),
                &mut self.vsync_enabled,
                &mut self.recreate_render_pass_manager,
            );

            if vsync_enabled != self.vsync_enabled {
                need_reconfigure = true;
            }

            self.egui_renderer.end_frame_and_draw(
                &self.device,
                &self.queue,
                &mut encoder,
                &self.window,
                &surface_view,
                screen_descriptor,
            );

            self.queue.submit(Some(encoder.finish()));
        }

        surface_texture.present();

        if need_reconfigure {
            self.set_vsync_enabled(self.vsync_enabled);
        }
        self.window.request_redraw();
    }

    pub fn event(&mut self, event: &WindowEvent) {
        self.egui_renderer.handle_input(&self.window, event);
    }
}
