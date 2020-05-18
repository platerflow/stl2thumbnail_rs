use crate::parser::Parser;
use std::io::{Read, Seek};

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

// Mesh
pub type Mesh = Vec<Triangle>;

// glm aliases
pub type Mat4 = glm::Mat4x4;
pub type Vec2 = glm::Vec2;
pub type Vec3 = glm::Vec3;
pub type Vec4 = glm::Vec4;

// helpers
pub fn matmul(m: &Mat4, v: &Vec3) -> Vec3 {
    (m * &Vec4::new(v.x, v.y, v.z, 1.0)).xyz()
}

// LazyMesh
pub struct LazyMesh<'a, T: Read + Seek> {
    parser: &'a mut Parser<T>,
}

impl<'a, T> LazyMesh<'a, T>
where
    T: Read + Seek,
{
    pub fn new(parser: &'a mut Parser<T>) -> Self {
        Self { parser }
    }
}

pub struct LazyMeshIter<'a, T: Read + Seek> {
    lazy_mesh: &'a mut LazyMesh<'a, T>,
}

impl<'a, T: Read + Seek> IntoIterator for &'a mut LazyMesh<'a, T> {
    type Item = Triangle;
    type IntoIter = LazyMeshIter<'a, T>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.parser.rewind();
        Self::IntoIter { lazy_mesh: self }
    }
}

impl<T: Read + Seek> Iterator for LazyMeshIter<'_, T> {
    type Item = Triangle;

    fn next(&mut self) -> Option<Self::Item> {
        self.lazy_mesh.parser.next_triangle()
    }
}

pub struct LazyMeshIter2<'a, T: Read + Seek> {
    lazy_mesh: LazyMesh<'a, T>,
}

impl<T: Read + Seek> Iterator for LazyMeshIter2<'_, T> {
    type Item = Triangle;

    fn next(&mut self) -> Option<Self::Item> {
        self.lazy_mesh.parser.next_triangle()
    }
}


impl<'a, T: Read + Seek> IntoIterator for LazyMesh<'a,T> {
    type Item = Triangle;
    type IntoIter = LazyMeshIter2<'a, T>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.parser.rewind();
        Self::IntoIter { lazy_mesh: self }
    }
}
