use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct QRend {
    size: ScreenSize,
    bounds: Quad,
    quads: Vec<Quad>,
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind: wgpu::BindGroup,
    vertex_cap: usize,
    pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct Quad {
    pub color: [f32; 4],
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
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

/// vertex, index, instance
fn gen_buffers(
    device: &wgpu::Device,
    quads: &[Quad],
) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
    let (instance_bytes, vertices, indices) = gen_buffer_data(quads);
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("wgputris.qrend.vertex_buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("wgputris.qrend.index_buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
    });
    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("wgputris.qrend.instance_buffer"),
        contents: instance_bytes,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
    (vertex_buffer, index_buffer, instance_buffer)
}

fn gen_buffer_data(quads: &[Quad]) -> (&[u8], Vec<Vertex>, Vec<u16>) {
    #[rustfmt::skip]
    const VERTICES: [Vertex; 4] = [
        Vertex { pos: [-1.0, -1.0] }, // bottom-left
        Vertex { pos: [ 1.0, -1.0] }, // bottom-right
        Vertex { pos: [ 1.0,  1.0] }, // top-right
        Vertex { pos: [-1.0,  1.0] }, // top-left
    ];
    #[rustfmt::skip]
    const INDICES: [u16; 6] = [
        0, 1, 3,
        1, 2, 3,
    ];
    let instance_bytes = bytemuck::cast_slice(quads);
    let vertices: Vec<_> = std::iter::repeat_n(VERTICES, quads.len())
        .flatten()
        .collect();
    let indices: Vec<_> = std::iter::repeat_n(INDICES, quads.len())
        .enumerate()
        .flat_map(|(off, i)| i.map(|i| i + (VERTICES.len() * off) as u16))
        .collect();
    (instance_bytes, vertices, indices)
}

impl QRend {
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

    pub fn new(
        size: ScreenSize,
        bounds: Quad,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        surface: wgpu::Surface<'static>,
        count: usize,
    ) -> Self {
        let quads = Vec::new();
        let (vertex_buffer, index_buffer, instance_buffer) = gen_buffers(&device, &quads);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("wgputris.qrend.uniform.buffer"),
            contents: bytemuck::bytes_of(&size),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let (uniform_bind, uniform_layout) = uniform_binding(&device, &uniform_buffer);
        let pipeline = create_pipeline(&device, format, uniform_layout);
        println!("here");
        Self {
            size,
            bounds,
            quads,
            queue,
            device,
            surface,
            surface_format: format,
            vertex_cap: count,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            uniform_bind,
            pipeline,
        }
    }

    pub fn resize(&mut self, size: ScreenSize) {
        let bytes = bytemuck::bytes_of(&size);
        self.queue
            .write_buffer_with(
                &self.vertex_buffer,
                0,
                wgpu::BufferSize::new(bytes.len() as u64).expect("invalid byte length"),
            )
            .expect("invalid quad buffer size")
            .copy_from_slice(&bytes);
        self.size = size;
        self.configure_surface();
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        let quads = self.quads.len() as u32;
        render_pass.set_scissor_rect(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width,
            self.bounds.height,
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..quads, 0, 0..quads);
        render_pass.draw(0..(self.quads.len() as u32), 0..1);
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
        fn write_bytes(queue: &wgpu::Queue, buffer: &wgpu::Buffer, bytes: &[u8]) {
            queue
                .write_buffer_with(
                    buffer,
                    0,
                    wgpu::BufferSize::new(bytes.len() as u64).expect("invalid byte length"),
                )
                .expect("invalid quad buffer size")
                .copy_from_slice(bytes);
        }
        write_bytes(
            &self.queue,
            &self.uniform_buffer,
            bytemuck::bytes_of(&self.size),
        );
        if self.vertex_cap < self.quads.len() {
            (self.vertex_buffer, self.index_buffer, self.instance_buffer) =
                gen_buffers(&self.device, &self.quads);
            self.vertex_cap = self.quads.len();
        } else {
            let (instance_bytes, vertices, indices) = gen_buffer_data(&self.quads);
            let vertex_bytes = bytemuck::cast_slice(&vertices);
            let index_bytes = bytemuck::cast_slice(&indices);

            write_bytes(&self.queue, &self.vertex_buffer, vertex_bytes);
            write_bytes(&self.queue, &self.index_buffer, index_bytes);
            write_bytes(&self.queue, &self.instance_buffer, instance_bytes);

            self.queue.submit([]);
        }
    }

    pub fn set_bounds(&mut self, bounds: Quad) {
        self.bounds = bounds;
    }
}

fn uniform_binding(
    device: &wgpu::Device,
    uniform_buffer: &wgpu::Buffer,
) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("wgputris.qrend.uniform.bind_group.layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
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
    (uniform_bind, uniform_layout)
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array!(
        // Position
        0 => Float32x2,
    );

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl Quad {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array!(
        // Color
        1 => Float32x4,
        // Position
        2 => Float32x2,
        // Size
        3 => Float32x2,
    );

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
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
            buffers: &[Vertex::desc(), Quad::desc()],
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
            front_face: wgpu::FrontFace::Ccw,
            ..Default::default()
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
