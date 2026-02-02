#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use android_logger::Config;
    use log::LevelFilter;
    use winit::event_loop::{ControlFlow, EventLoop};
    use winit::platform::android::EventLoopBuilderExtAndroid;
    use zu_core::app_state::AppState;
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Info) // limit log level
            .with_tag("zu_engine"), // logs will show under mytag tag
    );

    log::info!("Starting engine on Android......");
    zu_core::start_puffin_server();
    let event_loop: EventLoop<AppState> = EventLoop::<AppState>::with_user_event()
        .with_android_app(app)
        .build()
        .expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = zu_core::app::App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app).expect("Failed to run app");
}
