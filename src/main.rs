use winit::event_loop::{ControlFlow, EventLoop};

// NOTE: voxel engine:
// keep two quad buckets, one for void-like, one normal.
// grow meshes by iterating over the void-like.

fn main() {
    // for some reason setting env vars doesn't work when compiling for windows
    // std::env::set_var("RUST_BACKTRACE", "full");
    std::env::set_var("RUST_LOG", "wgputris=info,wgpu=error");
    env_logger::init_from_env(env_logger::Env::new()); // NOTE: can't use tracing, must use log
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll); // fast rendering
    let mut app = wgputris::App::default();
    event_loop.run_app(&mut app).unwrap();
}

// event_loop.set_control_flow(ControlFlow::Wait); // idle rendering
