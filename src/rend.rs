use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct QRend {
    size: ScreenSize,
    pub quads: Vec<Quad>,
    quad_cap: usize,
    // TODO: seperate buffers, rendering, and other wgpu
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct Quad {
    pub colour: [f32; 4],
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub color: [f32; 4],
    pub x: u32,
    pub y: u32,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct ScreenSize {
    width: u32,
    height: u32,
}

impl From<winit::dpi::PhysicalSize<u32>> for ScreenSize {
    fn from(winit::dpi::PhysicalSize { width, height }: winit::dpi::PhysicalSize<u32>) -> Self {
        Self { width, height }
    }
}

const UNIFORM_SIZE: std::num::NonZero<u64> =
    wgpu::BufferSize::new(std::mem::size_of::<ScreenSize>() as u64).unwrap();
impl QRend {
    pub fn new(
        size: ScreenSize,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        surface: wgpu::Surface<'static>,
        count: usize,
    ) -> Self {
        let quad_cap = count * 6 * std::mem::size_of::<Vertex>();
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wgputris.qrend.vertex_buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: quad_cap as u64,
            mapped_at_creation: false,
        });
        let (uniform_bind, uniform_layout, uniform_buffer) = uniform_binding(&device, size);
        let pipeline = create_pipeline(&device, format, uniform_layout);
        let this = Self {
            size,
            quads: Vec::new(),
            queue,
            device,
            surface,
            surface_format: format,
            quad_cap,
            vertex_buffer,
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
            .copy_from_slice(&bytes);
        self.size = size;
        self.configure_surface();
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        if self.vertex_buffer.size() == 0 {
            return;
        }
        // log::info!("rendering, vertices: {vertices}");
        let vertices = 6 * self.quads.len() as u32;
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..vertices, 0..1);
    }

    pub fn push(&mut self, quad: Quad) {
        self.quads.push(quad);
    }

    pub fn set(&mut self, index: usize, quad: Quad) {
        if let Some(q) = self.quads.get_mut(index) {
            *q = quad;
        }
    }

    pub fn prepare(&mut self) {
        if self.quad_cap * 6 * std::mem::size_of::<Vertex>() < self.quads.len() {
            self.vertex_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("wgputris.qrend.vertex_buffer"),
                        contents: bytemuck::cast_slice(&Vertex::from_quads(&self.quads)),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
            self.quad_cap = self.quads.len() * 6 * std::mem::size_of::<Vertex>();
        } else {
            // log::info!("writing vertex");
            let vertices = Vertex::from_quads(&self.quads);
            let bytes = bytemuck::cast_slice(&vertices);
            self.queue
                .write_buffer_with(
                    &self.vertex_buffer,
                    0,
                    wgpu::BufferSize::new(bytes.len() as u64).expect("invalid byte length"),
                )
                .expect("invalid quad buffer size")
                .copy_from_slice(bytes);
            self.queue.submit([]);
        }
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

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array!(
        // Color
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
            colour: color,
            x,
            y,
            width,
            height,
        }: &Quad,
    ) -> [Self; 6] {
        let vertex = |x, y| Vertex { color, x, y };
        let bl = vertex(x, y + height);
        let br = vertex(x + width, y + height);
        let tr = vertex(x + width, y);
        let tl = vertex(x, y);
        [tl, bl, br, tr, tl, br]
    }

    fn from_quads<'a>(quads: &'a [Quad]) -> Vec<Self> {
        let mut vertices = Vec::with_capacity(quads.len() * 6);
        for v in quads.iter().map(Self::from_quad) {
            vertices.extend_from_slice(&v);
        }
        vertices
    }
}

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
                blend: Some(wgpu::BlendState::REPLACE),
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
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}
