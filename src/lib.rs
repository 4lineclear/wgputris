pub mod draw;
pub mod game;
pub mod rend;
pub mod styling;

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
    settings: styling::Settings,
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

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let mut rend = rend::QRend::new(size.into(), device, queue, surface_format, surface);
        let game = Game::new();
        let settings = styling::Settings::default();

        let mut base_layer = rend.create_layer();
        let mut game_layer = rend.create_layer();

        draw::base_quads(&settings, &game, &mut base_layer, true);
        draw::game_quads(&settings, &game, &mut game_layer, true);
        rend.push_layer("base", base_layer);
        rend.push_layer("game", game_layer);
        State {
            game,
            rend,
            window,
            settings,
        }
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.settings
                .sizing
                .resize(&self.game, new_size.width, new_size.height);
            if let Some(layer) = self.rend.get_layer_mut("game") {
                draw::game_quads(&self.settings, &self.game, layer, false);
            }
            if let Some(layer) = self.rend.get_layer_mut("base") {
                draw::base_quads(&self.settings, &self.game, layer, false);
            }
            self.rend.resize(new_size.into());
        }
    }

    fn handle_key(&self, event: winit::event::KeyEvent) {
        use winit::keyboard::KeyCode::*;
        let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key else {
            return;
        };
        match key_code {
            ArrowLeft => {}
            ArrowRight => {}
            ArrowUp => {}
            ArrowDown => {}
            _ => (),
        }
    }

    fn render(&mut self) {
        self.rend.prepare();
        let output = self.rend.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.rend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("command encoder"),
                });
        self.rend
            .render(&mut encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("wgputris.render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: f64::from(self.settings.palette.bg.r),
                            g: f64::from(self.settings.palette.bg.g),
                            b: f64::from(self.settings.palette.bg.b),
                            a: f64::from(self.settings.palette.bg.a),
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            }));
        self.rend.queue.submit([encoder.finish()]);
        output.present();
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
        self.state = Some(pollster::block_on(State::new(window.clone())));
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().expect("state missing");
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                state.resize(size); // always followed by a redraw request
            }
            WindowEvent::KeyboardInput { event, .. } => {
                state.handle_key(event);
            }
            _ => (),
        }
    }
}
