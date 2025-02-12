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

pub const STARTING_VERTEX_COUNT: usize = game::TOTAL_BLOCKS as usize * 6;

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
        let game = Game::new();
        let mut rend = rend::QRend::new(
            size.into(),
            device,
            queue,
            surface_format,
            surface,
            STARTING_VERTEX_COUNT,
        );
        let settings = styling::Settings::default();
        for quad in game_quads(&settings, &game) {
            rend.push(quad);
        }
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
            write_game_quads(&self.settings, &self.game, &mut self.rend.quads);
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
                state.rend.prepare();
                let output = state.rend.surface.get_current_texture().unwrap();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    state
                        .rend
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("command encoder"),
                        });
                state
                    .rend
                    .render(&mut encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("wgputris.render_pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    }));
                state.rend.queue.submit([encoder.finish()]);
                output.present();
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

fn game_quads(settings: &styling::Settings, game: &game::Game) -> Vec<rend::Quad> {
    // let n_wide = game.board().line(0usize).blocks().len() as u32;
    // let n_tall = game.board().visible().len() as u32;
    // let board_width = (block_gap + block_size) * n_wide;
    // let board_height = (block_gap + block_size) * n_tall;
    // vec![quad(styling.bg, 0, 0, board_width, board_height)]
    let mut quads = vec![rend::Quad::default(); 200];
    write_game_quads(settings, game, &mut quads[..]);
    quads
}

fn write_game_quads(
    styling::Settings {
        styling,
        sizing:
            styling::Sizing {
                game_x,
                game_y,
                block_size,
                block_gap,
            },
    }: &styling::Settings,
    game: &game::Game,
    quads: &mut [rend::Quad],
) {
    fn quad(colour: styling::Colour, x: u32, y: u32, width: u32, height: u32) -> rend::Quad {
        rend::Quad {
            colour: colour.rgba(),
            x,
            y,
            width,
            height,
        }
    }
    let mut i = 0;
    let mut cx = *game_x;
    let mut cy = *game_y;
    for line in game.board().visible() {
        cy += block_gap;
        for &b in line.blocks() {
            cx += block_gap;
            quads[i] = quad(styling.colour_block(b), cx, cy, *block_size, *block_size);
            cx += block_size;
            i += 1;
        }
        cy += block_size;
        cx = *game_x;
    }
}
