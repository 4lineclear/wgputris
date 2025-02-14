// use std::sync::Arc;

use winit::event_loop::{ControlFlow, EventLoop};

// NOTE: voxel engine:
// keep two quad buckets, one for void-like, one normal.
// grow meshes by iterating over the void-like.

//
fn main() {
    #[cfg(debug_assertions)]
    setup_logging();

    let mut app = wgputris::App::new();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll); // fast rendering
    event_loop.run_app(&mut app).unwrap();
}

// event_loop.set_control_flow(ControlFlow::Wait); // idle rendering

// for some reason setting env vars doesn't work when compiling for windows
// so we have this instead
#[cfg(debug_assertions)]
fn setup_logging() {
    std::env::set_var("RUST_LOG", "wgputris=info,wgpu=error");
    env_logger::init_from_env(env_logger::Env::new());
}
