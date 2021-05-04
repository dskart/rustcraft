#[macro_use]
extern crate slog;
extern crate slog_term;

use anyhow::Result;
use clap::{App, Arg};
use slog::Drain;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

mod atlas;
mod bitmap_font;
mod block;
mod camera;
mod chunk;
mod coordinate;
mod crosshair;
mod engine;
mod framerate;
mod noise;
mod overlay_info;
mod player;
mod ray_tracer;
mod renderer;
mod texture;
mod world;
mod world_generation;

use engine::*;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub wireframe: bool,
    pub display_coordinates: bool,
    pub flat_world: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wireframe: false,
            display_coordinates: false,
            flat_world: false,
        }
    }
}

impl Config {
    fn new(matches: clap::ArgMatches) -> Self {
        let mut config = Config::default();
        if matches.is_present("WIREFRAME") {
            config.wireframe = true;
        }
        if matches.is_present("COORDINATES") {
            config.display_coordinates = true;
        }
        if matches.is_present("FLATWORLD") {
            config.flat_world = true;
        }
        return config;
    }
}

fn setup_logger() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());
    return logger;
}

fn event_loop(logger: slog::Logger, config: Config) -> Result<()> {
    let event_loop = EventLoop::new();
    let title = env!("CARGO_PKG_NAME");
    let start_size = winit::dpi::LogicalSize::new(1000.0, 800.0);
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .with_inner_size(start_size)
        .build(&event_loop)?;
    window.set_cursor_visible(false);
    // window.set_cursor_grab(true)?;

    use futures::executor::block_on;
    let renderer = block_on(renderer::Renderer::new(logger.clone(), &window))?;
    let mut engine = Engine::new(logger.clone(), config, renderer)?;
    engine.resize(start_size.to_physical(1.0));

    let mut last_render_time = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::DeviceEvent { ref event, .. } => {
                engine.input(event);
            }
            Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                if !engine.input_keyboard(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => {
                                *control_flow = ControlFlow::Exit;
                            }
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            engine.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            engine.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                engine.update(dt);
                match engine.render() {
                    Ok(_) => {}
                    Err(err) => match err.downcast_ref::<wgpu::SwapChainError>() {
                        // Recreate the swap_chain if lost
                        Some(wgpu::SwapChainError::Lost) => engine.resize(engine.renderer.size),
                        // The system is out of memory, we should probably quit
                        Some(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        _ => error!(logger, "{:?}", err),
                    },
                }
            }
            _ => {}
        }
    });
}

fn main() -> Result<()> {
    let logger = setup_logger();

    let matches = App::new("RustCraft")
        .version("1.0")
        .about("Creates a minecraft like engine")
        .arg(
            Arg::with_name("WIREFRAME")
                .short("w")
                .long("wireframe")
                .required(false)
                .takes_value(false)
                .help("Renders polygons in wireframe mode"),
        )
        .arg(
            Arg::with_name("COORDINATES")
                .short("c")
                .long("coordinates")
                .required(false)
                .takes_value(false)
                .help("Renders coordinate system"),
        )
        .arg(
            Arg::with_name("FLATWORLD")
                .short("f")
                .long("flat_world")
                .required(false)
                .takes_value(false)
                .help("Generates a flat world"),
        )
        .get_matches();

    let config = Config::new(matches);

    event_loop(logger, config)?;

    Ok(())
}
