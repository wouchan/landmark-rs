#[derive(Debug, Clone, Copy, Default)]
pub struct Transform {
    pub rotation: glam::Quat,
    pub translation: glam::Vec3,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct RawTransform(glam::Mat4);

impl RawTransform {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl From<Transform> for RawTransform {
    fn from(value: Transform) -> Self {
        let rot_mat = glam::Mat4::from_quat(value.rotation);
        let pos_mat = glam::Mat4::from_translation(value.translation);

        let mat = pos_mat * rot_mat;

        Self(mat)
    }
}
