use std::f32::consts::PI;
use std::mem;
use std::path::Path;

use cocoa::appkit::NSView;
use cocoa::base::YES;
use core_graphics_types::geometry::CGSize;
use glam::Mat4;
use metal::{Device, MetalLayer, MTLPixelFormat, RenderPipelineState, RenderPipelineDescriptor, CommandQueue, Library, MTLResourceOptions, RenderPassDescriptor, MTLClearColor, MTLStoreAction, MTLScissorRect, MTLPrimitiveType, MTLViewport, Buffer, TextureDescriptor, MTLRegion, MTLSize, MTLOrigin, DepthStencilDescriptor, MTLCompareFunction, DepthStencilState};
use metal::foreign_types::ForeignType;
use winit::platform::macos::WindowExtMacOS;
use metal::MTLLoadAction;
use winit::window::Window;

use crate::mesh::{Mesh, Model};
use crate::structs::{Vertex, ConstBuffer, Transform};
use crate::texture::Texture;

// Todo: add transform
pub struct ModelQueueEntry {
    pub model_id: usize,
    pub transform: Transform,
}

pub struct Renderer{
    pub device: Option<Device>,
    pipeline_state: Option<RenderPipelineState>,
    library: Option<Library>,
    command_queue: Option<CommandQueue>,
    layer: Option<MetalLayer>,
    const_buffer_gpu: Vec<Buffer>,
    const_buffer_cpu: ConstBuffer,
    loaded_models: Vec<Model>,
    loaded_textures: Vec<metal::Texture>,
    model_queue: Vec<ModelQueueEntry>,
    depth_texture: Option<metal::Texture>,
    depth_stencil_state: Option<DepthStencilState>,
    tex_white: usize,
}

impl Renderer{
    pub fn new(window: &Window) -> Self {
        // Initialize renderer with none
        let mut renderer = Renderer {
            device: None,
            pipeline_state: None,
            command_queue: None,
            library: None,
            layer: None,
            const_buffer_cpu: ConstBuffer{
                model_matrix: Mat4::IDENTITY,
                view_matrix: Mat4::IDENTITY,
                proj_matrix: Mat4::IDENTITY,
            },
            const_buffer_gpu: Vec::new(),
            model_queue: Vec::new(),
            loaded_models: Vec::new(),
            loaded_textures: Vec::new(),
            depth_texture: None,
            depth_stencil_state: None,
            tex_white: 0,
        };

        // Create device
        renderer.device = Some(Device::system_default().expect("Could not create device."));

        // Create metal layer
        renderer.layer = Some(MetalLayer::new());
        renderer.layer.as_ref().unwrap().set_device(renderer.device.as_ref().unwrap());
        renderer.layer.as_ref().unwrap().set_pixel_format(MTLPixelFormat::RGBA8Unorm);
        renderer.layer.as_ref().unwrap().set_presents_with_transaction(false);

        // Create view - a sort of canvas where you draw graphics using Metal commands
        unsafe {
            let view = window.ns_view() as cocoa::base::id;
            view.setWantsLayer(YES);
            view.setLayer(mem::transmute(renderer.layer.as_ref().unwrap().as_ptr()));
        }

        // Store the view size
        let drawable_size = window.inner_size();

        // Resize framebuffer
        renderer.resize_framebuffer(drawable_size.width, drawable_size.height);
        renderer.layer.as_ref().unwrap().set_drawable_size(CGSize::new(drawable_size.width as f64, drawable_size.height as f64));

        // Create command queue
        renderer.command_queue = Some(renderer.device.as_ref().unwrap().new_command_queue());

        // Initialize default white texture
        let tex_white = Some(Texture {
            gl_id: 0,
            width: 1,
            height: 1,
            depth: 1,
            data: vec![0xFFFFFFFFu32]
        });
        renderer.tex_white = renderer.upload_texture(&mut tex_white.unwrap());

        return renderer;
    }

    pub fn load_library(&mut self, path: &str) {    
        self.library = Some(self.device.as_ref().unwrap().new_library_with_file(path).expect("Failed to load Metal library"));
    }

    pub fn prepare_pipeline_state (
        &mut self,
        vertex_shader_path: &str,
        fragment_shader_path: &str,
    ) {
        // Get compiled functions from the library
        let vertex_function = self.library.as_ref().unwrap().get_function(vertex_shader_path, None).unwrap();
        let fragment_function = self.library.as_ref().unwrap().get_function(fragment_shader_path, None).unwrap();

        // Create pipeline state descriptor - handles things like shader program, buffer to render to, blend mode, etc.
        let pipeline_state_desc = RenderPipelineDescriptor::new();
        pipeline_state_desc.set_vertex_function(Some(&vertex_function));
        pipeline_state_desc.set_fragment_function(Some(&fragment_function));
        pipeline_state_desc.set_depth_attachment_pixel_format(MTLPixelFormat::Depth32Float);

        let color_attachment = pipeline_state_desc.color_attachments().object_at(0).unwrap();
        color_attachment.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
        color_attachment.set_blending_enabled(false);

        self.pipeline_state = Some(self.device.as_ref().unwrap().new_render_pipeline_state(&pipeline_state_desc).unwrap());

        let depth_stencil_desc = DepthStencilDescriptor::new();
        depth_stencil_desc.set_depth_write_enabled(true);
        depth_stencil_desc.set_depth_compare_function(MTLCompareFunction::Less);
        self.depth_stencil_state = Some(self.device.as_ref().unwrap().new_depth_stencil_state(&depth_stencil_desc));
    }

    pub fn upload_vertex_buffer(&mut self, mesh: &mut Mesh) {
        // Create the vertex buffer on the device
        mesh.buffer = Some(self.device.as_ref().unwrap().new_buffer_with_data(
            mesh.verts.as_ptr() as *const _,
            (mesh.verts.len() * mem::size_of::<Vertex>()) as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
        ));
    }

    pub fn begin_frame(&mut self) {
        self.model_queue.clear();
        self.const_buffer_gpu.clear();
    }

    pub fn end_frame(&mut self) {
        // Get the next framebuffer
        let layer = self.layer.as_ref().unwrap();
        let drawable = match layer.next_drawable() {
            Some(drawable) => drawable,
            None => return,
        };
        let size = self.layer.as_ref().unwrap().drawable_size();

        // Set up framebuffer
        let render_pass_descriptor = RenderPassDescriptor::new();
        let color_attachment = render_pass_descriptor.color_attachments().object_at(0).expect("Failed to get color attachment");
        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(MTLLoadAction::Clear);
        color_attachment.set_clear_color(MTLClearColor::new(0.1, 0.1, 0.2, 1.0));
        color_attachment.set_store_action(MTLStoreAction::Store);

        // Set up depth buffer
        let depth_attachment = render_pass_descriptor.depth_attachment().unwrap();
        depth_attachment.set_texture(Some(self.depth_texture.as_ref().unwrap()));
        depth_attachment.set_load_action(MTLLoadAction::Clear);
        depth_attachment.set_clear_depth(1.0);
        depth_attachment.set_store_action(MTLStoreAction::Store);

        // Set up command buffer
        let command_buffer = self.command_queue.as_ref().unwrap().new_command_buffer();
        let command_encoder = command_buffer.new_render_command_encoder(render_pass_descriptor);

        // Record mesh draw calls
        command_encoder.set_render_pipeline_state(self.pipeline_state.as_ref().unwrap().as_ref());
        command_encoder.set_depth_stencil_state(self.depth_stencil_state.as_ref().unwrap());
        command_encoder.set_cull_mode(metal::MTLCullMode::None);
        command_encoder.set_scissor_rect(MTLScissorRect{x: 0, y: 0, width: size.width as u64, height: size.height as u64});
        command_encoder.set_viewport(MTLViewport{
            originX: 0.0,
            originY: 0.0,
            width: size.width,
            height: size.height,
            znear: -1.0,
            zfar: 1.0,
        });
        for model_id in &self.model_queue {
            self.const_buffer_cpu.model_matrix = model_id.transform.local_matrix().transpose();
            self.const_buffer_gpu.push(self.device.as_ref().unwrap().new_buffer_with_data(
                &mut self.const_buffer_cpu as *mut _ as *const std::ffi::c_void,
                mem::size_of::<ConstBuffer>() as u64,
                MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
            ));
            command_encoder.set_vertex_buffer(1, Some(self.const_buffer_gpu.last().unwrap()), 0);
            Self::update_const_buffer_gpu(self.const_buffer_gpu.last_mut().unwrap(), &self.const_buffer_cpu);
            
            let model = &self.loaded_models[model_id.model_id];
            for name in model.meshes.keys() {
                let mesh = model.meshes.get(name).unwrap();
                let material = model.materials.get(name);
                if let Some(mat) = material {
                    command_encoder.set_fragment_texture(0, Some(&self.loaded_textures[mat.tex_alb as usize]));
                } else {
                    command_encoder.set_fragment_texture(0, Some(&self.loaded_textures[self.tex_white]));
                }
                command_encoder.set_vertex_buffer(0, Some(mesh.buffer.as_ref().unwrap()), 0);
                command_encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, mesh.verts.len() as u64);
            }
        }
        command_encoder.end_encoding();

        // Present framebuffer
        command_buffer.present_drawable(drawable);
        command_buffer.commit();
    }

    pub fn resize_framebuffer(&mut self, width: u32, height: u32) {
        println!("framebuffer resized to {width}, {height}");
        self.layer.as_ref().unwrap().set_drawable_size(CGSize::new(width as f64, height as f64));
        
        let depth_texture_desc = TextureDescriptor::new();
        depth_texture_desc.set_width(width as u64);
        depth_texture_desc.set_height(height as u64);
        depth_texture_desc.set_pixel_format(MTLPixelFormat::Depth32Float);
        self.depth_texture = Some(self.device.as_ref().unwrap().new_texture(&depth_texture_desc));
    }

    pub fn draw_model(&mut self, model_queue_entry: ModelQueueEntry) {
        self.model_queue.push(model_queue_entry);
    }

    pub fn load_model(&mut self, path: &Path) -> Option<usize> {
        let mut model = match Model::load_gltf(path, self) {
            Ok(mdl) => mdl,
            Err(s) => {println!("Error loading model \"{}\": {s}", path.display()); return None;}
        };
        
        for (name, mesh) in &mut model.meshes {
            println!("Uploading mesh {name}");
            self.upload_vertex_buffer(mesh);
        }

        self.loaded_models.push(model);
        return Some(self.loaded_models.len() - 1);
    }

    pub fn update_camera(&mut self, camera_transform: &Transform) {
        // Update CPU-side buffer
        self.const_buffer_cpu.view_matrix = camera_transform.view_matrix().transpose();
        self.const_buffer_cpu.proj_matrix = Mat4::perspective_rh(PI / 4.0, 16.0 / 9.0, 0.1, 1000.0).transpose();
    }

    fn update_const_buffer_gpu(buffer_gpu: &mut Buffer, buffer_cpu: &ConstBuffer){
        let buffer_gpu_data = buffer_gpu.contents();
        unsafe {
            std::ptr::copy(buffer_cpu, buffer_gpu_data as *mut ConstBuffer, 1);
        }
    }

    pub fn upload_texture(&mut self, texture: &mut Texture) -> usize {
        let texture_desc = TextureDescriptor::new();
        texture_desc.set_width(texture.width as u64);
        texture_desc.set_height(texture.height as u64);
        texture_desc.set_pixel_format(MTLPixelFormat::RGBA8Unorm);

        let texture_gpu = self.device.as_ref().unwrap().new_texture(&texture_desc);
        texture_gpu.replace_region(MTLRegion{
            origin: MTLOrigin { x: 0, y: 0, z: 0 },
            size: MTLSize {
                width: texture.width as u64,
                height: texture.height as u64,
                depth: 1,
            },
        }, 0, texture.data.as_ptr() as _, texture.width as u64 * 4);
        texture.gl_id = self.loaded_textures.len() as u32;
        self.loaded_textures.push(texture_gpu);
        return self.loaded_textures.len() - 1;
    }
}