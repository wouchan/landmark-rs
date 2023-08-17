use wgpu::util::DeviceExt;

use crate::rendererer::Renderer;

#[derive(Debug)]
pub struct Camera {
    eye: glam::Vec3,
    target: glam::Vec3,
    up: glam::Vec3,
    aspect: f32,
    fovy: f32,
    near: f32,
    view_proj: glam::Mat4,
    pub buffer: wgpu::Buffer,
}

impl Camera {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let eye = glam::Vec3::new(0.0, 0.0, -1.0);
        let target = glam::Vec3::ZERO;
        let up = glam::Vec3::Y;
        let aspect = config.width as f32 / config.height as f32;
        let fovy: f32 = 75.0;
        let near = 0.1;

        let view = glam::Mat4::look_at_rh(eye, target, up);
        let proj = glam::Mat4::perspective_infinite_rh(fovy.to_radians(), aspect, near);

        let view_proj = proj * view;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[view_proj]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            near,
            view_proj,
            buffer,
        }
    }

    pub fn update_view_projection_matrix(&mut self, renderer: &Renderer) {
        self.aspect = renderer.config.width as f32 / renderer.config.height as f32;

        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj =
            glam::Mat4::perspective_infinite_rh(self.fovy.to_radians(), self.aspect, self.near);

        self.view_proj = proj * view;

        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.view_proj]));
    }
}
