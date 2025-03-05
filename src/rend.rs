use std::rc::Rc;
use std::sync::Mutex;

use bytemuck::{Pod, Zeroable};
use indexmap::IndexMap;
use wgpu::util::DeviceExt;

use crate::styling::Colour;

pub use self::quad_layer::QuadLayer;
pub use self::text_layer::{TextLayer, TextLayerDesc};

pub mod quad_layer;
pub mod text_layer;

#[derive(Debug)]
pub struct Rend {
    size: ScreenSize,
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    uniform_buffer: wgpu::Buffer,
    uniform_bind: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    qrend: QRend,
    trend: TRend,
}

struct TRend {
    font_system: Rc<Mutex<glyphon::FontSystem>>,
    swash_cache: glyphon::SwashCache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,
    layers: IndexMap<&'static str, TextLayer>,
}

#[derive(Debug, Default)]
pub struct QRend {
    layers: IndexMap<&'static str, QuadLayer>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Quad {
    pub colour: Colour,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub colour: [f32; 4],
    pub x: u32,
    pub y: u32,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct ScreenSize {
    width: u32,
    height: u32,
    scale: f64,
}

impl ScreenSize {
    pub fn new(size: winit::dpi::PhysicalSize<u32>, scale: f64) -> Self {
        Self {
            width: size.width,
            height: size.height,
            scale,
        }
    }
}

const UNIFORM_SIZE: std::num::NonZero<u64> =
    wgpu::BufferSize::new(std::mem::size_of::<ScreenSize>() as u64).unwrap();

impl Rend {
    pub fn new(
        size: ScreenSize,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        surface: wgpu::Surface<'static>,
    ) -> Self {
        let (uniform_bind, uniform_layout, uniform_buffer) = uniform_binding(&device, size);
        let pipeline = create_pipeline(&device, format, uniform_layout);
        let this = Self {
            size,
            qrend: QRend::default(),
            trend: TRend::new(&device, &queue, format),
            queue,
            device,
            surface,
            surface_format: format,
            uniform_buffer,
            uniform_bind,
            pipeline,
        };
        this.configure_surface();
        this
    }

    pub fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, size: ScreenSize) {
        let bytes = bytemuck::bytes_of(&size);
        self.queue
            .write_buffer_with(&self.uniform_buffer, 0, UNIFORM_SIZE)
            .expect("invalid quad buffer size")
            .copy_from_slice(bytes);
        self.size = size;
        self.configure_surface();
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind, &[]);
        self.qrend.render(render_pass);
        self.trend.render(&self.size, &self.queue, render_pass);
    }

    pub fn create_quad_layer(&self, name: &'static str) -> QuadLayer {
        QuadLayer::new(name, "wgputris.rend.layer", &self.device, 0)
    }

    pub fn push_quad_layer(&mut self, layer: QuadLayer) {
        self.qrend.layers.insert(layer.name(), layer);
    }

    pub fn gen_quad_layer(&mut self, name: &'static str) {
        self.push_quad_layer(self.create_quad_layer(name));
    }

    pub fn create_text_layer(
        &mut self,
        metrics: glyphon::Metrics,
        desc: TextLayerDesc,
    ) -> TextLayer {
        let buffer = glyphon::Buffer::new(&mut self.trend.font_system.lock().unwrap(), metrics);
        TextLayer::new(buffer, desc, self.trend.font_system.clone())
    }

    pub fn push_text_layer(&mut self, layer: TextLayer) {
        self.trend.layers.insert(layer.name(), layer);
    }

    pub fn gen_text_layer(&mut self, metrics: glyphon::Metrics, desc: TextLayerDesc) {
        let layer = self.create_text_layer(metrics, desc);
        self.push_text_layer(layer);
    }

    pub fn get_quad_mut(&mut self, label: &'static str) -> Option<&mut QuadLayer> {
        self.qrend.layers.get_mut(label)
    }
    pub fn get_text_mut(&mut self, label: &'static str) -> Option<&mut TextLayer> {
        self.trend.layers.get_mut(label)
    }

    pub fn prepare(&mut self) {
        for (_, layer) in &mut self.qrend.layers {
            layer.prepare(&self.device, &self.queue);
        }
        self.trend.prepare(&self.device, &self.queue);
    }

    pub fn finish(&mut self) {
        self.trend.finish();
    }
}

fn uniform_binding(
    device: &wgpu::Device,
    size: ScreenSize,
) -> (wgpu::BindGroup, wgpu::BindGroupLayout, wgpu::Buffer) {
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("wgputris.qrend.uniform.buffer"),
        contents: bytemuck::bytes_of(&size),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("wgputris.qrend.uniform.bind_group.layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(UNIFORM_SIZE),
            },
            count: None,
        }],
    });
    let uniform_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("wgputris.qrend.uniform.bind_group"),
        layout: &uniform_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    (uniform_bind, uniform_layout, uniform_buffer)
}

pub const VERTICES_PER_QUAD: usize = 6;
pub const BYTES_PER_QUAD: usize = VERTICES_PER_QUAD * std::mem::size_of::<Vertex>();

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array!(
        // Colour
        0 => Float32x4,
        // Position + Size
        1 => Uint32x2,
    );

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }

    fn from_quad(
        &Quad {
            colour,
            x,
            y,
            width,
            height,
        }: &Quad,
    ) -> [Self; 6] {
        let colour = colour.rgba();
        let vertex = |x, y| Vertex { colour, x, y };
        let bl = vertex(x, y + height);
        let br = vertex(x + width, y + height);
        let tr = vertex(x + width, y);
        let tl = vertex(x, y);
        [tl, bl, br, tr, tl, br]
    }

    fn from_quads(quads: &[Quad]) -> Vec<Self> {
        let mut vertices = Vec::with_capacity(quads.len() * 6);
        for v in quads.iter().map(Self::from_quad) {
            vertices.extend_from_slice(&v);
        }
        vertices
    }
}

const MULTISAMPLE_STATE: wgpu::MultisampleState = wgpu::MultisampleState {
    count: 1,
    mask: !0,
    alpha_to_coverage_enabled: false,
};

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_bind: wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("wgputtris.qrend.pipeline_layout"),
        push_constant_ranges: &[],
        bind_group_layouts: &[&uniform_bind],
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("wgputtris.qrend.shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
            "./shaders/quad.wgsl"
        ))),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("wgputtris.qrend.pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MULTISAMPLE_STATE,
        multiview: None,
        cache: None,
    })
}

impl QRend {
    fn render(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        for layer in self.layers.values().filter(|l| !l.is_empty()) {
            layer.render(render_pass)
        }
    }
}

impl TRend {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        swapchain_format: wgpu::TextureFormat,
    ) -> TRend {
        use glyphon::*;
        let font_system = Rc::new(Mutex::new(FontSystem::new()));
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, swapchain_format);
        let text_renderer = TextRenderer::new(&mut atlas, &device, MULTISAMPLE_STATE, None);

        // let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 26.0));
        // let physical_width = (size.width as f64 * size.scale) as f32;
        // let physical_height = (size.height as f64 * size.scale) as f32;
        //
        // text_buffer.set_size(
        //     &mut font_system,
        //     Some(physical_width),
        //     Some(physical_height),
        // );
        // const TEXT: &str = "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z";
        // text_buffer.set_text(
        //     &mut font_system,
        //     TEXT,
        //     Attrs::new()
        //         .family(Family::SansSerif)
        //         .color(glyphon::Color::rgba(255, 255, 255, 255)),
        //     Shaping::Advanced,
        // );
        // text_buffer.shape_until_scroll(&mut font_system, false);

        TRend {
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            layers: IndexMap::default(),
        }
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system.lock().unwrap(),
                &mut self.atlas,
                &self.viewport,
                self.layers.values().map(TextLayer::to_area),
                &mut self.swash_cache,
            )
            .unwrap();
    }

    pub fn finish(&mut self) {
        self.atlas.trim();
    }

    fn render(
        &mut self,
        size: &ScreenSize,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.viewport.update(
            &queue,
            glyphon::Resolution {
                width: size.width,
                height: size.height,
            },
        );
        self.text_renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }
}

impl std::fmt::Debug for TRend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TRend")
            .field("font_system", &self.font_system)
            .field("swash_cache", &self.swash_cache)
            .field("viewport", &self.viewport)
            .field("atlas", &())
            .field("text_renderer", &())
            .field("layers", &self.layers)
            .finish()
    }
}
