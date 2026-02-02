use log::info;
use std::env;
use winit::event_loop::{ControlFlow, EventLoop};
use zu_core::{app::App, start_puffin_server};

#[cfg(target_os = "windows")]
use win_dialog::WinDialog;

#[cfg(target_os = "windows")]
use backtrace::Backtrace;

#[cfg(target_os = "windows")]
use std::panic::{self, PanicInfo};

/// Parse CLI arguments and return module path if specified
fn parse_args() -> Option<String> {
    let args: Vec<String> = env::args().collect();

    // Skip program name
    let mut args_iter = args.iter().skip(1);

    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "-m" | "--module" => {
                return args_iter.next().cloned();
            }
            "--help" | "-h" => {
                println!("Zurie Engine");
                println!();
                println!("Usage: zu_core [OPTIONS] [MODULE_PATH]");
                println!();
                println!("Options:");
                println!("  -m, --module <PATH>  Path to WASM module to load");
                println!("  -h, --help           Show this help message");
                println!();
                println!("Examples:");
                println!("  zu_core                                    # Use engine.toml or show picker");
                println!("  zu_core game.wasm                          # Load game.wasm");
                println!("  zu_core -m ./target/wasm32-wasip2/release/my_game.wasm");
                std::process::exit(0);
            }
            path if !path.starts_with('-') => {
                return Some(path.to_string());
            }
            _ => {}
        }
    }

    None
}

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        extern crate console_error_panic_hook;
        use std::panic;
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Trace);
    }
    #[cfg(target_os = "windows")]
    {
        // Set the custom panic hook only on Windows
        panic::set_hook(Box::new(|panic_info: &PanicInfo| {
            // Capture panic details
            let message = format!(
                "Panic occurred!\n\nMessage: {}\nLocation: {}\n\nBacktrace:\n{:?}",
                panic_info
                    .payload()
                    .downcast_ref::<&str>()
                    .unwrap_or(&"Unknown"),
                panic_info
                    .location()
                    .unwrap_or(&std::panic::Location::caller()),
                Backtrace::new()
            );

            // Display Win32 MessageBox dialog via win_dialog
            let _ = WinDialog::new(&message)
                .with_header("Application Panic")
                .show();
        }));
    }
    start_puffin_server();

    // Parse CLI arguments
    let cli_module = parse_args();
    if let Some(ref path) = cli_module {
        info!("CLI module path: {}", path);
    }

    info!("Starting App");
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run(cli_module));
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(run(None));
    }
}

async fn run(cli_module: Option<String>) {
    let event_loop = EventLoop::with_user_event().build().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(
        cli_module,
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );

    event_loop.run_app(&mut app).expect("Failed to run app");
}
