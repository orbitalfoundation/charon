
use crossbeam::channel::*;
use service::*;


use std::iter;
use std::error::Error;
use std::{cell::RefCell, collections::HashMap};

use cgmath::prelude::*;

use winit::dpi::LogicalSize;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use futures::future::Future;
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, GlyphBrush, Region, Section, Text};
use wgpu::util::DeviceExt;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;





///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

mod camera;
mod model;
mod texture; // NEW!

use model::{DrawLight, DrawModel, Vertex};

const NUM_INSTANCES_PER_ROW: u32 = 10;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    // UPDATED!
    fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into()
    }
}

struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl model::Vertex for InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Light {
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,
    obj_model: model::Model,
    camera: camera::Camera,                      // UPDATED!
    projection: camera::Projection,              // NEW!
    camera_controller: camera::CameraController, // UPDATED!
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    instances: Vec<Instance>,
    #[allow(dead_code)]
    instance_buffer: wgpu::Buffer,
    depth_texture: texture::Texture,
    size: winit::dpi::PhysicalSize<u32>,
    light: Light,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    debug_material: model::Material,
    // NEW!
    mouse_pressed: bool,

    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
    local_spawner: futures::executor::LocalSpawner,
    mybrush: RefCell<GlyphBrush<()>>,


}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(&shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{:?}", shader)),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrite::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLAMPING
            clamp_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    })
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                    // normal map
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // UPDATED!
        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection =
            camera::Projection::new(sc_desc.width, sc_desc.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera, &projection);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = cgmath::Vector3 { x, y: 0.0, z };

                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(
                            position.clone().normalize(),
                            cgmath::Deg(45.0),
                        )
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let now = std::time::Instant::now();
        let obj_model = model::Model::load(
            &device,
            &queue,
            &texture_bind_group_layout,
            "view/res/cube.obj",
        )
        .unwrap();
        println!("Elapsed (Original): {:?}", std::time::Instant::now() - now);

        let light = Light {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &uniform_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                sc_desc.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                shader,
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                sc_desc.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
            )
        };

        let debug_material = {
            let diffuse_bytes = include_bytes!("../res/cobble-diffuse.png");
            let normal_bytes = include_bytes!("../res/cobble-normal.png");

            let diffuse_texture = texture::Texture::from_bytes(
                &device,
                &queue,
                diffuse_bytes,
                "res/alt-diffuse.png",
                false,
            )
            .unwrap();
            let normal_texture = texture::Texture::from_bytes(
                &device,
                &queue,
                normal_bytes,
                "res/alt-normal.png",
                true,
            )
            .unwrap();

            model::Material::new(
                &device,
                "alt-material",
                diffuse_texture,
                normal_texture,
                &texture_bind_group_layout,
            )
        };



		// Create staging belt and a local pool
		let staging_belt = wgpu::util::StagingBelt::new(1024);
		let local_pool = futures::executor::LocalPool::new();
		let local_spawner = local_pool.spawner();

		// Prepare glyph_brush // Result<(ab_glyph::FontArc), Box<dyn Error>>
		//let inconsolata = ab_glyph::FontArc::try_from_slice(include_bytes!("Inconsolata-Regular.ttf"));

		//let mut glyph_brush = GlyphBrushBuilder::using_font(inconsolata).build(&device, sc_desc.format );

		// Prepare glyph_brush
		let inconsolata = ab_glyph::FontArc::try_from_slice(include_bytes!(
			"Inconsolata-Regular.ttf"
		)).unwrap();

		let mybrush = RefCell::new( GlyphBrushBuilder::using_font(inconsolata).build(&device, sc_desc.format) );


        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline,
            obj_model,
            camera,
            projection,
            camera_controller,
            uniform_buffer,
            uniform_bind_group,
            uniforms,
            instances,
            instance_buffer,
            depth_texture,
            size,
            light,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
            #[allow(dead_code)]
            debug_material,
            // NEW!
            mouse_pressed: false,

            staging_belt,
            local_pool,
            local_spawner,
            mybrush,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // UPDATED!
        self.projection.resize(new_size.width, new_size.height);
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
    }

    fn input(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            }) => self.camera_controller.process_keyboard(*key, *state),
            DeviceEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            DeviceEvent::Button {
                button: 1, // Left Mouse Button
                state,
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_pressed {
                    self.camera_controller.process_mouse(delta.0, delta.1);
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.uniforms.update_view_proj(&self.camera, &self.projection);

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        // Update the light
        let old_position: cgmath::Vector3<_> = self.light.position.into();
        self.light.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();

        self.queue
            .write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light]));
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder"), });

        // Render Pass 3d

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.uniform_bind_group,
                &self.light_bind_group,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                &self.uniform_bind_group,
                &self.light_bind_group,
            );
        }

        // Render Pass 2d - font test

		self.mybrush.borrow_mut().queue(Section {
			screen_position: (30.0, 30.0),
			bounds: (self.size.width as f32, self.size.height as f32),
			text: vec![Text::new("ORBITAL ORBITAL")
				.with_color([0.0, 1.0, 0.0, 1.0])
				.with_scale(40.0)],
			..Section::default()
		});

		self.mybrush.borrow_mut().queue(Section {
			screen_position: (30.0, 90.0),
			bounds: (self.size.width as f32, self.size.height as f32),
			text: vec![Text::new("ORBITTTTTT")
				.with_color([1.0, 0.5, 1.0, 1.0])
				.with_scale(40.0)],
			..Section::default()
		});

	  // Draw the text!
		self.mybrush.borrow_mut().draw_queued_with_transform_and_scissoring(
			&self.device,
			&mut self.staging_belt,
			&mut encoder,
			&frame.view,
			wgpu_glyph::orthographic_projection(
				self.size.width,
				self.size.height,
			),
			Region {
				x: 20,
				y: 100,
				width: 200,
				height: 35,
			},
		).expect("Draw queued");


		// Fancy Finish ... from font landia

		// Submit the work!
		self.staging_belt.finish();
		self.queue.submit(Some(encoder.finish()));

		// Recall unused staging buffers
		use futures::task::SpawnExt;

		// wait
		self.local_spawner.spawn(self.staging_belt.recall()).expect("Recall staging belt");
		self.local_pool.run_until_stalled();

        // self.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}

fn runstuff()  -> Result<(), Box<dyn Error>>  {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Orbital")
        .build(&event_loop)
        .unwrap();
    let mut state = pollster::block_on(State::new(&window)); // NEW!
    let mut last_render_time = std::time::Instant::now();



    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::DeviceEvent {
                ref event,
                .. // We're not using device_id currently
            } => {
                state.input(event);
            }
            // UPDATED!
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            // UPDATED!
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);
                match state.render() {
                    Ok(_) => {


                    }
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    });
}












































///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////



///
/// A renderable thing
///

struct Renderable {
	kind: i16,
	box_x: i16,
	box_y: i16,
	velocity_x: i16,
	velocity_y: i16,
}

impl Renderable {

	/// Update the internal state; bounce the box around the screen.
	fn update(&mut self) {
		if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
			self.velocity_x *= -1;
		}
		if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
			self.velocity_y *= -1;
		}

		self.box_x += self.velocity_x;
		self.box_y += self.velocity_y;
	}

	/// Draw the state to the frame buffer.
	///
	/// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
	fn draw(&self, frame: &mut [u8]) {
//        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
//            let x = (i % WIDTH as usize) as i16;
//            let y = (i / WIDTH as usize) as i16;
//        }
	}
}


///
/// A 3d engine more or less, opens a window, renders a scene graph, handles input events.
/// This is tied to my messaging system so it can receive commands to fiddle with the scene graph.
///

#[derive(Clone)]
pub struct View {}


impl View {
	pub fn new() -> Box<dyn Serviceable> {
		Box::new(Self{})
	}

	fn start2(&self, _name: String, _sid: SID, send: Sender<Message>, recv: Receiver<Message> ) -> Result<(), Box<dyn Error>>  {

		let _send = send.clone();
		let _recv = recv.clone();
		let _name = self.name();

		//////////////////////////////////////////////////////////////////////////////////////////////////////////////
		// This is an array of things to render

		let mut objects: Vec<Renderable> = vec![];

		// clear backgrounder
		objects.push( Renderable {
			kind: 0,
			box_x: 24,
			box_y: 16,
			velocity_x: 1,
			velocity_y: 1,
		});

		//////////////////////////////////////////////////////////////////////////////////////////////////////////////
		// build a display

		let mut input = WinitInputHelper::new();

		let event_loop = EventLoop::new();

		let window = {
			let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
			WindowBuilder::new()
				.with_title("Orbital")
				.with_inner_size(size)
				.with_min_inner_size(size)
				.build(&event_loop)
				.unwrap()
		};

		let window_size = window.inner_size();

		let instance = wgpu::Instance::new(wgpu::BackendBit::all());
		let surface = unsafe { instance.create_surface(&window) };

		// Initialize GPU - uses futures -> see https://github.com/hecrj/wgpu_glyph/blob/master/examples/hello.rs
		let (device, queue) = futures::executor::block_on(async {
			let adapter = instance
				.request_adapter(&wgpu::RequestAdapterOptions {
					power_preference: wgpu::PowerPreference::HighPerformance,
					compatible_surface: Some(&surface),
				})
				.await
				.expect("Request adapter");

			adapter
				.request_device(&wgpu::DeviceDescriptor::default(), None)
				.await
				.expect("Request device")
		});


		// Create staging belt and a local pool
		let mut staging_belt = wgpu::util::StagingBelt::new(1024);
		let mut local_pool = futures::executor::LocalPool::new();
		let local_spawner = local_pool.spawner();

		// Prepare swap chain
		let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
		let mut size = window.inner_size();

		let mut swap_chain = device.create_swap_chain(
			&surface,
			&wgpu::SwapChainDescriptor {
				usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
				format: render_format,
				width: size.width,
				height: size.height,
				present_mode: wgpu::PresentMode::Mailbox,
			},
		);

		// Prepare glyph_brush
		let inconsolata = ab_glyph::FontArc::try_from_slice(include_bytes!(
			"Inconsolata-Regular.ttf"
		))?;

		let mut glyph_brush = GlyphBrushBuilder::using_font(inconsolata).build(&device, render_format);

		// Render loop
		window.request_redraw();

		event_loop.run(move |event, _, control_flow| {
			match event {

				/////////////////////////////////////////////////////////////////////////////////////////////////////
				// window event close
				/////////////////////////////////////////////////////////////////////////////////////////////////////
				winit::event::Event::WindowEvent {
					event: winit::event::WindowEvent::CloseRequested,
					..
				} => *control_flow = winit::event_loop::ControlFlow::Exit,

				/////////////////////////////////////////////////////////////////////////////////////////////////////
				// window event resize
				/////////////////////////////////////////////////////////////////////////////////////////////////////
				winit::event::Event::WindowEvent {
					event: winit::event::WindowEvent::Resized(new_size),
					..
				} => {
					size = new_size;

					swap_chain = device.create_swap_chain(
						&surface,
						&wgpu::SwapChainDescriptor {
							usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
							format: render_format,
							width: size.width,
							height: size.height,
							present_mode: wgpu::PresentMode::Mailbox,
						},
					);
				}

				/////////////////////////////////////////////////////////////////////////////////////////////////////
				// window event redraw
				/////////////////////////////////////////////////////////////////////////////////////////////////////
				winit::event::Event::RedrawRequested { .. } => {
					// Get a command encoder for the current frame
					let mut encoder = device.create_command_encoder(
						&wgpu::CommandEncoderDescriptor {
							label: Some("Redraw"),
						},
					);

					// Get the next frame
					let frame = swap_chain
						.get_current_frame()
						.expect("Get next frame")
						.output;

					// Clear frame
					{
						let _ = encoder.begin_render_pass(
							&wgpu::RenderPassDescriptor {
								label: Some("Render pass"),
								color_attachments: &[
									wgpu::RenderPassColorAttachment {
										view: &frame.view,
										resolve_target: None,
										ops: wgpu::Operations {
											load: wgpu::LoadOp::Clear(
												wgpu::Color {
													r: 0.4,
													g: 0.4,
													b: 0.4,
													a: 1.0,
												},
											),
											store: true,
										},
									},
								],
								depth_stencil_attachment: None,
							},
						);
					}

					glyph_brush.queue(Section {
						screen_position: (30.0, 30.0),
						bounds: (size.width as f32, size.height as f32),
						text: vec![Text::new("Hello wgpu_glyph!")
							.with_color([0.0, 0.0, 0.0, 1.0])
							.with_scale(40.0)],
						..Section::default()
					});

					glyph_brush.queue(Section {
						screen_position: (30.0, 90.0),
						bounds: (size.width as f32, size.height as f32),
						text: vec![Text::new("Hello wgpu_glyph!")
							.with_color([1.0, 1.0, 1.0, 1.0])
							.with_scale(40.0)],
						..Section::default()
					});

				  // Draw the text!
					glyph_brush
						.draw_queued_with_transform_and_scissoring(
							&device,
							&mut staging_belt,
							&mut encoder,
							&frame.view,
							wgpu_glyph::orthographic_projection(
								size.width,
								size.height,
							),
							Region {
								x: 40,
								y: 105,
								width: 200,
								height: 15,
							},
						)
						.expect("Draw queued");

					// Submit the work!
					staging_belt.finish();
					queue.submit(Some(encoder.finish()));

					// Recall unused staging buffers
					use futures::task::SpawnExt;

					local_spawner
						.spawn(staging_belt.recall())
						.expect("Recall staging belt");

					local_pool.run_until_stalled();
				}

				/////////////////////////////////////////////////////////////////////////////////////////////////////
				// other
				/////////////////////////////////////////////////////////////////////////////////////////////////////

				_ => {
					*control_flow = winit::event_loop::ControlFlow::Wait;
				}
			}
		})
	}

}


impl Serviceable for View {
	fn name(&self) -> &str { "View" }
	fn stop(&self) {}
	fn start(&self, _name: String, _sid: SID, send: Sender<Message>, recv: Receiver<Message> ) {

		runstuff();

		//self.start2(_name,_sid,send,recv);

/*
		//////////////////////////////////////////////////////////////////////////////////////////////////////////////
		// tell the system message broker that I want to listen for messages to '/display'
		// TODO - i wonder if the broker cannot be smarter? think about wiring a bit more
		// TODO - also, I could just tell the message system what my receive port is at this time; not earlier
		// TOOD - or... send to this based on absolute path /system/context/display or something?

		let message = Message::Subscribe(_sid,"/display".to_string());
		send.send(message).expect("error");

		//////////////////////////////////////////////////////////////////////////////////////////////////////////////
		// Run display forever

		event_loop.run(move |event, _, control_flow| {


			// TODO
			//		- catch messages here and then add new objects to some kind of display list
			//		- i could test with 2d boxes
			//		- how hard is it to tell wgpu to make and cache say a box, or text, or a 3d object?
			//		- how hard is it to dynamically extend or add new things to the list of wgpu objects?
			//		- it is a big hassle to do camera projection, shader lighting transforms and so on?
			//		- is it a big hassle to load fonts
			//		- is it a big hassle to load gltf?
			//		- see https://nyxtom.dev/2020/10/08/framebuffers/

			while let Ok(message) = recv.try_recv() {
				match message {
					Message::Event(topic,data) => {
						println!("View: Received: {} {}",topic, data);
						match data.as_str() {
							"cube" => {
								println!("Display: got a cube");
								let r = Renderable {
									kind: 1,
									box_x: 24,
									box_y: 16,
									velocity_x: 1,
									velocity_y: 1,
								};
								objects.push(r);
							},
							_ => {
							}
						}
					},
					//Message::Share(sharedmemory) => {
					//},
					_ => { },
				}
			}

			// Draw the current frame
			if let Event::RedrawRequested(_) = event {

				for i in 0..objects.len() {

					let r = &mut objects[i];

					//r.draw(pixels.get_frame());

					r.update();
				}

				//if pixels.render().is_err() {
				//	*control_flow = ControlFlow::Exit;
				//	return;
				//}

			}

			// Handle input events
			if input.update(&event) {
				// Close events
				if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
					*control_flow = ControlFlow::Exit;
					return;
				}

				// Resize the window
				if let Some(size) = input.window_resized() {
					//pixels.resize_surface(size.width, size.height);
				}

				// Redraw
				window.request_redraw();
			}
		});
*/

	}
}


