use shipyard::*;
use wgpu::util::DeviceExt;

use crate::rendererer::Renderer;

#[derive(Debug, Unique)]
pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fovy: f32,
    aspect: f32,
    near: f32,
    view_proj: glam::Mat4,
    pub buffer: wgpu::Buffer,
}

impl Camera {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let eye = glam::Vec3::new(0.0, 0.0, -1.0);
        let target = glam::Vec3::ZERO;
        let aspect = config.width as f32 / config.height as f32;
        let fovy: f32 = 75.0;
        let near = 0.1;

        let view = glam::Mat4::look_at_lh(eye, target, glam::Vec3::Y);
        let proj = glam::Mat4::perspective_infinite_lh(fovy.to_radians(), aspect, near);

        let view_proj = proj * view;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[view_proj]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            eye,
            target,
            yaw: 0.0,
            pitch: 0.0,
            fovy,
            aspect,
            near,
            view_proj,
            buffer,
        }
    }

    pub fn update_view_projection_matrix(&mut self, renderer: &Renderer) {
        self.aspect = renderer.config.width as f32 / renderer.config.height as f32;

        let mut look_direction = glam::Vec3::Z;
        look_direction = glam::Mat3::from_rotation_x(self.pitch.to_radians()) * look_direction;
        look_direction = glam::Mat3::from_rotation_y(self.yaw.to_radians()) * look_direction;
        look_direction = look_direction.normalize();

        self.target = self.eye + look_direction;

        let view = glam::Mat4::look_at_lh(self.eye, self.target, glam::Vec3::Y);
        let proj =
            glam::Mat4::perspective_infinite_lh(self.fovy.to_radians(), self.aspect, self.near);

        self.view_proj = proj * view;

        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.view_proj]));
    }
}

pub fn update_camera_sys(mut camera: UniqueViewMut<Camera>, renderer: UniqueView<Renderer>) {
    camera.update_view_projection_matrix(&renderer);
}
