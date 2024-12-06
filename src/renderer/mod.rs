use std::{fs::File, sync::Arc};

use context::WgpuContext;
use futures::executor::block_on;
use pipelines::Pipelines;

use crate::scene::{PrimitiveBatch, Scene};

pub mod context;
mod gpu_vec;
mod pipelines;

#[repr(C)]
#[derive(Clone, Copy)]
struct GlobalParams {
    viewport_size: [f32; 2],
    premultiplied_alpha: u32,
    pub pad: u32, // align to 8 bytes
}

pub struct Renderer {
    gpu_ctx: Arc<WgpuContext>,
    pipelines: Pipelines,
}

impl Renderer {
    pub fn new(gpu_ctx: Arc<WgpuContext>) -> Self {
        let pipelines = Pipelines::new(&gpu_ctx);

        Self { gpu_ctx, pipelines }
    }

    // TODO: render_pass should generate every frame instead of being passed in
    pub fn draw(&mut self, scene: &Scene) {
        let device = &self.gpu_ctx.device;
        let queue = &self.gpu_ctx.queue;
        let texture_size = wgpu::Extent3d {
            width: 512,
            height: 512,
            depth_or_array_layers: 1,
        };

        let texture_desc = wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            label: None,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        };

        let render_texture = device.create_texture(&texture_desc);

        let view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let pass_descriptor = wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        };

        let global_params = GlobalParams {
            viewport_size: [texture_size.width as f32, texture_size.height as f32],
            premultiplied_alpha: 0,
            pad: 0,
        };

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = command_encoder.begin_render_pass(&pass_descriptor);

            for batch in scene.batches() {
                match batch {
                    PrimitiveBatch::Quads(quads) => {
                        let pipeline_ctx = &mut self.pipelines.quads;

                        pipeline_ctx.update(device, queue, global_params, quads);

                        render_pass.set_pipeline(&pipeline_ctx.pipeline);

                        render_pass.set_bind_group(0, &pipeline_ctx.bind_group, &[]);

                        render_pass.draw(0..4, 0..quads.len() as u32);
                    }
                }
            }
        }
        queue.submit(Some(command_encoder.finish()));

        save(device, queue, &texture_desc, &render_texture);
    }
}

// TODO: delete this when we have a window
fn save(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    descriptor: &wgpu::TextureDescriptor,
    texture: &wgpu::Texture,
) {
    let texture_extent = descriptor.size;

    let bytes_per_piexel = match descriptor.format {
        wgpu::TextureFormat::Rgba8UnormSrgb => 4,
        wgpu::TextureFormat::Rgba8Unorm => 1,
        _ => panic!("unsupported pixel format"),
    };

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (texture_extent.width * texture_extent.height * bytes_per_piexel) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    assert!((texture_extent.width * bytes_per_piexel) % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0);

    let command_encoder = {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(texture_extent.width * bytes_per_piexel),
                    rows_per_image: None,
                },
            },
            texture_extent,
        );

        encoder.finish()
    };

    queue.submit(Some(command_encoder));

    device.poll(wgpu::Maintain::Wait);

    block_on(create_png("demo.png", device, output_buffer, descriptor))
}

pub async fn create_png(
    png_output_path: &str,
    device: &wgpu::Device,
    output_buffer: wgpu::Buffer,
    texture_descriptor: &wgpu::TextureDescriptor<'_>,
) {
    let texture_extent = texture_descriptor.size;

    // Note that we're not calling `.await` here.
    let buffer_slice = output_buffer.slice(..);

    // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    device.poll(wgpu::Maintain::Wait);
    // If a file system is available, write the buffer as a PNG
    let has_file_system_available = cfg!(not(target_arch = "wasm32"));
    if !has_file_system_available {
        return;
    }

    if let Some(Ok(())) = receiver.receive().await {
        let buffer_view = buffer_slice.get_mapped_range();

        let mut png_encoder = png::Encoder::new(
            File::create(png_output_path).unwrap(),
            texture_extent.width,
            texture_extent.height,
        );
        png_encoder.set_depth(png::BitDepth::Eight);
        png_encoder.set_color(match texture_descriptor.format {
            wgpu::TextureFormat::Rgba8UnormSrgb => png::ColorType::Rgba,
            wgpu::TextureFormat::R8Unorm => png::ColorType::Grayscale,
            _ => panic!("unsupported pixel format"),
        });
        let mut png_writer = png_encoder.write_header().unwrap();

        png_writer.write_image_data(&buffer_view).unwrap();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(buffer_view);

        output_buffer.unmap();
    }
}
