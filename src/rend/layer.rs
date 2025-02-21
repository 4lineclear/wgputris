#[derive(Debug)]
pub struct Layer {
    label: &'static str,
    quads: Vec<super::Quad>,
    buffer: wgpu::Buffer,
    byte_cap: usize,
    changed: bool,
}

impl Layer {
    pub fn new(label: &'static str, device: &wgpu::Device, quads: usize) -> Self {
        let byte_cap = quads * super::BYTES_PER_QUAD;
        Self {
            label,
            quads: Vec::with_capacity(quads),
            buffer: create_buffer(label, device, byte_cap),
            byte_cap,
            changed: false,
        }
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.changed {
            return;
        }
        let vertices = super::Vertex::from_quads(&self.quads);
        let contents = bytemuck::cast_slice(&vertices);
        let byte_len = contents.len();
        if self.byte_cap < byte_len {
            self.buffer = create_buffer(self.label, device, byte_len);
            self.byte_cap = byte_len;
        } else if let Some(mut size) = wgpu::BufferSize::new(byte_len as u64)
            .and_then(|size| queue.write_buffer_with(&self.buffer, 0, size))
        {
            size.copy_from_slice(contents);
        }
    }

    pub fn byte_cap(&self) -> usize {
        self.byte_cap
    }

    pub fn len(&self) -> usize {
        self.quads.len()
    }

    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
    }

    pub fn set_quads(&mut self, quads: Vec<super::Quad>) {
        self.changed = true;
        self.quads = quads;
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn vertices(&self) -> usize {
        self.quads.len() * super::VERTICES_PER_QUAD
    }

    pub fn with_quads(label: &'static str, device: &wgpu::Device, quads: Vec<super::Quad>) -> Self {
        use wgpu::util::DeviceExt;
        let vertices = super::Vertex::from_quads(&quads);
        let contents = bytemuck::cast_slice(&vertices);
        let byte_cap = contents.len();
        Self {
            label,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }),
            quads,
            byte_cap,
            changed: false,
        }
    }
}

fn create_buffer(label: &str, device: &wgpu::Device, byte_cap: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: byte_cap as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
