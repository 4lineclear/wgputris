pub mod game;
pub mod rend;

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::game::Game;

pub struct State {
    game: Game,
    rend: rend::QRend,
    window: Arc<Window>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl State {
    pub async fn new(window: Arc<Window>) -> State {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();
        let surface = instance.create_surface(window.clone()).unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let rend = rend::QRend::new(
            size.into(),
            rend::Quad {
                color: [0.0; 4],
                x: 0,
                y: 0,
                width: size.width,
                height: size.height,
            },
            device,
            queue,
            surface_format,
            surface,
            0,
        );
        rend.configure_surface();
        State {
            game: Game::default(),
            rend,
            window,
        }
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.rend.resize(new_size.into());
    }
}

#[derive(Default)]
pub struct App {
    pub state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        window.set_maximized(true);

        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().expect("state missing");
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.rend.prepare();
                let queue = state.rend.queue.clone();
                let mut encoder =
                    state
                        .rend
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("command encoder"),
                        });
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("wgputris.render_pass"),
                        color_attachments: &[],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    state.rend.render(&mut render_pass);
                }
                queue.submit([encoder.finish()]);
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                state.resize(size); // always followed by a redraw request
            }
            _ => (),
        }
    }
}
