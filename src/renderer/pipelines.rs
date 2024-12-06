use crate::scene::Quad;

use super::{
    context::WgpuContext,
    gpu_vec::{GPUVec, INIT_CAPACITY},
    GlobalParams,
};

pub(crate) struct PipelineCtx<T: Copy> {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub global_params_buffer: GPUVec<GlobalParams>,
    pub data_buffer: GPUVec<T>,
}

impl<T: Copy> PipelineCtx<T> {
    fn new(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        color_targets: &[Option<wgpu::ColorTargetState>],
        label: &str,
    ) -> Self {
        let global_params_buffer = GPUVec::<GlobalParams>::new_uniforms(device, "global_params");
        let data_buffer = GPUVec::<T>::new(device, INIT_CAPACITY, "data");
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(format!("{label}s bind group layout").as_str()),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                GPUVec::<T>::bind_group_layout_entry(1),
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("{label}s bind group").as_str()),
            layout: &bind_group_layout,
            entries: &[
                global_params_buffer.bind_group_entry(0),
                data_buffer.bind_group_entry(1),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{label}s pipeline layout").as_str()),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(format!("{label}s pipeline").as_str()),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some(format!("vs_{label}").as_str()),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(format!("fs_{label}").as_str()),
                targets: color_targets,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            bind_group_layout,
            global_params_buffer,
            data_buffer,
        }
    }

    pub(crate) fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        global_params: GlobalParams,
        data: &[T],
    ) {
        self.global_params_buffer
            .update(device, queue, &[global_params]);

        if self.data_buffer.update(device, queue, data) {
            self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("data bind group"),
                layout: &self.bind_group_layout,
                entries: &[
                    self.global_params_buffer.bind_group_entry(0),
                    self.data_buffer.bind_group_entry(1),
                ],
            });
        }
    }
}

pub(crate) struct Pipelines {
    pub quads: PipelineCtx<Quad>,
}

impl Pipelines {
    pub fn new(gpu_ctx: &WgpuContext) -> Self {
        let shader = gpu_ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "shader.wgsl"
                ))),
            });

        let color_targets = &[Some(wgpu::ColorTargetState {
            write_mask: wgpu::ColorWrites::default(),
            // TODO: texture format should get from a surface
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        })];

        Self {
            quads: PipelineCtx::new(&gpu_ctx.device, &shader, color_targets, "quad"),
        }
    }
}
