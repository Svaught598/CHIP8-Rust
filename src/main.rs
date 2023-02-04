#![deny(clippy::all)]
#![forbid(unsafe_code)]
mod chip_machine;
mod processor;

use chip_machine::CHIPMachine;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent },
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() -> Result<(), Error> {
    println!("START");
    env_logger::init();
    let event_loop = EventLoop::new();

    // setup window
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 10.0, HEIGHT as f64 * 10.0);
        WindowBuilder::new()
            .with_title("CHIP-8  Emulator")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    // setup pixel buffer
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let mut chip8: CHIPMachine = CHIPMachine::new(WIDTH as usize, HEIGHT as usize);
    chip8.load_rom(String::from("./roms/test_opcode.ch8"));
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    let key = input.virtual_keycode.unwrap();
                    chip8.process_key(key);
                }
                WindowEvent::CloseRequested => {
                    println!("Window close event detected");
                    *control_flow = ControlFlow::Exit
                },
                WindowEvent::Resized(size) => {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        error!("pixels.resize_surface() failed: {err}");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    window.request_redraw();
                }
                _ => ()
            },
            Event::MainEventsCleared => {
                // chip8 processing 
                let elapsed = chip8.start_time.elapsed();
                if elapsed >= chip8.cycle_duration {
                    chip8.cycle();
                    chip8.reset_start_time();
                }
                window.request_redraw();
            },
            Event::RedrawRequested(_) => {
                chip8.draw(pixels.get_frame_mut());
                if let Err(err) = pixels.render() {
                    error!("pixels.render() failed: {}", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            },
            _ => ()
        }
    });
}
