use crate::render_passes::render_pass_manager::RenderOptions;
use crate::widgets::usage_diagnostics::UsageDiagnostics;
use egui::Context;
use egui::Widget;
use egui_probe::Probe;
use puffin_egui::profiler_window;

pub struct EngineGui {
    egui_context: Context,
    open_profiler_window: bool,
}

impl EngineGui {
    pub fn new(context: &Context) -> Self {
        Self {
            egui_context: context.clone(),
            open_profiler_window: false,
        }
    }

    pub fn render_gui(
        &mut self,
        render_options: &mut RenderOptions,
        vsync_enabled: &mut bool,
        recreate_render_pass_manager: &mut bool,
    ) {
        egui::Window::new("Engine Window").show(&self.egui_context, |ui| {
            Probe::new(render_options).show(ui);
            UsageDiagnostics {}.ui(ui);
            ui.checkbox(vsync_enabled, "Vsync enabled");
            ui.checkbox(&mut self.open_profiler_window, "Open profiler window");
            *recreate_render_pass_manager = ui.button("Recreate Render Pass Manager").clicked();
        });
        if self.open_profiler_window {
            profiler_window(&self.egui_context);
        }
    }
}
