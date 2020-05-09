use std::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
    pub normal: Vec3,
}

impl Triangle {
    pub fn new(vertices: [Vec3; 3], normal: Vec3) -> Self {
        Self { vertices, normal }
    }
}

// aliases
pub type Mat4 = glm::Mat4x4;
pub type Vec2 = glm::Vec2;
pub type Vec3 = glm::Vec3;
pub type Vec4 = glm::Vec4;
pub type Mesh = Vec<Triangle>;

// helpers
pub fn matmul(m: &Mat4, v: &Vec3) -> Vec3 {
    (m * &Vec4::new(v.x, v.y, v.z, 1.0)).xyz()
}
