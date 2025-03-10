pub mod draw;
pub mod game;
pub mod key;
pub mod rend;
pub mod styling;
pub mod time;

use std::sync::{
    mpsc::{self},
    Arc, Mutex,
};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use self::rend::ScreenSize;

const RUNNING_ORDER: std::sync::atomic::Ordering = std::sync::atomic::Ordering::Relaxed;

/// External actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Hold,
    Place,
    Rotate180,
    RotateLeft,
    RotateRight,
    MoveRight,
    MoveLeft,
    MoveDown,
    Exit,
}

impl Action {
    pub fn repeatable(&self) -> bool {
        use Action::*;
        matches!(self, MoveRight | MoveLeft | MoveDown)
    }
}

pub struct State {
    // TODO: use an overarching 'GameState' struct instead of directly
    // handling the game struct.
    rend: rend::Rend,
    keys: mpsc::Sender<key::SentKey>,
    game: Arc<Mutex<game::Game>>,
    settings: styling::Settings,
    ctx: Arc<Context>,
    // NOTE: should be dropped last
    window: Arc<Window>,
}

struct Context {
    run: AtomicRunState,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            run: AtomicRunState::new(RunState::Running),
        }
    }
}

#[atomic_enum::atomic_enum]
enum RunState {
    Running,
    EndScheduled,
    Ended,
}

impl RunState {
    fn running(&self) -> bool {
        matches!(self, Self::Running)
    }
    fn ended(&self) -> bool {
        matches!(self, Self::Ended)
    }
}

impl State {
    async fn new(
        window: Arc<Window>,
        keys: mpsc::Sender<key::SentKey>,
        game: Arc<Mutex<game::Game>>,
        ctx: Arc<Context>,
    ) -> State {
        let size = window.inner_size();
        let scale = window.scale_factor();
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
        let mut rend = rend::Rend::new(
            ScreenSize::new(size, scale),
            device,
            queue,
            surface_format,
            surface,
        );

        rend.gen_text_layer(
            glyphon::Metrics {
                font_size: 24.0,
                line_height: 36.0,
            },
            rend::TextLayerDesc {
                name: "text",
                attrs: None,
                shaping: None,
                left: 0.0,
                top: 0.0,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: size.width as i32,
                    bottom: size.height as i32,
                },
                default_color: Some(glyphon::Color::rgb(0, 0, 0)),
                custom_glyphs: Vec::new(),
            },
        );
        rend.gen_quad_layer("base");
        rend.gen_quad_layer("game");

        State {
            rend,
            keys,
            game,
            window,
            settings: styling::Settings::default(),
            ctx,
        }
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.settings.sizing.resize(new_size.width, new_size.height);
            self.rend
                .resize(ScreenSize::new(new_size, self.window.scale_factor()));
            self.draw();
        }
    }

    fn draw(&mut self) {
        if let Some(layer) = self.rend.get_quad_mut("game") {
            draw::game_quads(&self.settings, &self.game.lock().unwrap(), layer);
        }
        if let Some(layer) = self.rend.get_quad_mut("base") {
            draw::base_quads(&self.settings, layer);
        }
        if let Some(layer) = self.rend.get_text_mut("text") {
            layer.set_text("Hello, World!");
        }
    }

    fn render(&mut self) {
        self.draw();
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::default()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            }));
        self.rend.queue.submit([encoder.finish()]);
        output.present();
        self.rend.finish();
    }
}

pub struct App {
    pub state: Option<State>,
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_visible(false)
                        .with_maximized(true)
                        .with_title("wgputris"),
                )
                .unwrap(),
        );
        let (sender, receiver) = mpsc::channel();
        let game: Arc<Mutex<game::Game>> = Default::default();
        let ctx: Arc<Context> = Arc::default();

        self.state = Some(pollster::block_on(State::new(
            window.clone(),
            sender,
            game.clone(),
            ctx.clone(),
        )));

        window.set_visible(true);
        window.focus_window();
        window.request_redraw();

        game_thread(window, receiver, game, ctx);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if event_loop.exiting() {
            return;
        }
        let state = self.state.as_mut().expect("state missing");
        let run_state = state.ctx.run.load(RUNNING_ORDER);
        if !run_state.running() {
            if !run_state.ended() {
                state.ctx.run.store(RunState::Ended, RUNNING_ORDER);
                event_loop.exit();
            }
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
            }
            WindowEvent::Resized(size) => {
                state.resize(size); // always followed by a redraw request
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(sk) = key::SentKey::from_event(event) {
                    state.keys.send(sk).unwrap();
                }
            }
            _ => (),
        }
    }
}

fn game_thread(
    window: Arc<Window>,
    keyr: mpsc::Receiver<key::SentKey>,
    game: Arc<Mutex<game::Game>>,
    ctx: Arc<Context>,
) {
    use std::ops::ControlFlow;
    let keys = key::KeyStore::default();
    time::run(
        move |action, _| {
            let mut game = game.lock().unwrap();
            for key in keyr.try_iter() {
                if let Some((action, pressed)) = keys.apply_key(key.key, key.pressed) {
                    if action == Action::Exit {
                        ctx.run.store(RunState::EndScheduled, RUNNING_ORDER);
                        return ControlFlow::Break(());
                    }
                    game.apply_action(action, pressed);
                }
            }
            for _ in 0..action.ticks {
                game.tick(action.now);
                for action in keys.get_actions() {
                    game.apply_action(action, true);
                }
            }
            if ctx.run.load(RUNNING_ORDER).ended() {
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        },
        move |_, _| {
            window.request_redraw();
        },
        120,
    );
}
