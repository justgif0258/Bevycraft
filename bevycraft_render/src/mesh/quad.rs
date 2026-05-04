use crate::prelude::{TextureId, Vertex};
use bevy::math::EulerRot;
use bevy::prelude::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Not;

pub const NEUTRAL_TINT: [f32; 4] = [1.0; 4];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    vertices: [Vertex; 4],
    render_mode: RenderMode,
    tintable: bool,
}

impl Quad {
    #[inline]
    pub fn new(
        min: [f32; 3],
        max: [f32; 3],
        uv: [f32; 4],
        texture: TextureId,
        mode: RenderMode,
        tintable: bool,
        facing: Direction,
    ) -> Self {
        Self {
            vertices: build_vertex_array(min, max, uv, texture, facing),
            render_mode: mode,
            tintable,
        }
    }

    #[inline]
    pub fn rotate(&mut self, origin: Vec3, x: f32, y: f32, z: f32) {
        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            x.to_radians(),
            y.to_radians(),
            z.to_radians(),
        );

        for vertex in self.vertices.iter_mut() {
            let pos = Vec3::from(vertex.position);
            let rotated = rotation * (pos - origin) + origin;

            vertex.position = rotated.into();

            let n = Vec3::from(vertex.normal);

            vertex.normal = (rotation * n).into();
        }
    }

    #[inline(always)]
    pub const fn positions(&self) -> [[f32; 3]; 4] {
        [
            self.vertices[0].position,
            self.vertices[1].position,
            self.vertices[2].position,
            self.vertices[3].position,
        ]
    }

    #[inline(always)]
    pub fn min(&self) -> [f32; 3] {
        self.vertices[0].position
    }

    #[inline(always)]
    pub fn max(&self) -> [f32; 3] {
        self.vertices[2].position
    }

    #[inline(always)]
    pub const fn normals(&self) -> [[f32; 3]; 4] {
        [
            self.vertices[0].normal,
            self.vertices[1].normal,
            self.vertices[2].normal,
            self.vertices[3].normal,
        ]
    }

    #[inline(always)]
    pub const fn uvs(&self) -> [[f32; 2]; 4] {
        [
            self.vertices[0].uv,
            self.vertices[1].uv,
            self.vertices[2].uv,
            self.vertices[3].uv,
        ]
    }

    #[inline(always)]
    pub const fn textures(&self) -> [u32; 4] {
        [
            self.vertices[0].texture.0,
            self.vertices[1].texture.0,
            self.vertices[2].texture.0,
            self.vertices[3].texture.0,
        ]
    }

    #[inline(always)]
    pub const fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    #[inline(always)]
    pub const fn tintable(&self) -> bool {
        self.tintable
    }
}

#[derive(Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    #[serde(rename = "east")]
    PosX = 0,
    #[serde(rename = "west")]
    NegX = 1,

    #[serde(rename = "up")]
    PosY = 2,
    #[serde(rename = "down")]
    NegY = 3,

    #[serde(rename = "south")]
    PosZ = 4,
    #[serde(rename = "north")]
    NegZ = 5,
}

impl Direction {
    #[inline(always)]
    pub const fn get_normal(self) -> [f32; 3] {
        match self {
            Direction::PosX => [1.0, 0.0, 0.0],
            Direction::NegX => [-1.0, 0.0, 0.0],
            Direction::PosY => [0.0, 1.0, 0.0],
            Direction::NegY => [0.0, -1.0, 0.0],
            Direction::PosZ => [0.0, 0.0, 1.0],
            Direction::NegZ => [0.0, 0.0, -1.0],
        }
    }
}

impl Display for Direction {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::PosX => f.write_str("east"),
            Direction::NegX => f.write_str("west"),
            Direction::PosY => f.write_str("up"),
            Direction::NegY => f.write_str("down"),
            Direction::PosZ => f.write_str("south"),
            Direction::NegZ => f.write_str("north"),
        }
    }
}

impl Not for Direction {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        match self {
            Direction::PosX => Direction::NegX,
            Direction::NegX => Direction::PosX,
            Direction::PosY => Direction::NegY,
            Direction::NegY => Direction::PosY,
            Direction::PosZ => Direction::NegZ,
            Direction::NegZ => Direction::PosZ,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RenderMode {
    Opaque = 0,
    Cutout = 1,
    Translucent = 2,
}

impl Display for RenderMode {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderMode::Opaque => f.write_str("opaque"),
            RenderMode::Cutout => f.write_str("cutout"),
            RenderMode::Translucent => f.write_str("translucent"),
        }
    }
}

impl Default for RenderMode {
    #[inline(always)]
    fn default() -> Self {
        RenderMode::Opaque
    }
}

#[inline(always)]
const fn build_vertex_array(
    min: [f32; 3],
    max: [f32; 3],
    uv: [f32; 4],
    texture: TextureId,
    facing: Direction,
) -> [Vertex; 4] {
    let arranged_uvs = [
        [uv[0], uv[3]],
        [uv[2], uv[3]],
        [uv[2], uv[1]],
        [uv[0], uv[1]],
    ];

    let normal = facing.get_normal();

    match facing {
        Direction::PosX => [
            Vertex {
                position: [max[0], min[1], max[2]],
                uv: arranged_uvs[0],
                normal,
                texture,
            },
            Vertex {
                position: [max[0], min[1], min[2]],
                uv: arranged_uvs[1],
                normal,
                texture,
            },
            Vertex {
                position: [max[0], max[1], min[2]],
                uv: arranged_uvs[2],
                normal,
                texture,
            },
            Vertex {
                position: [max[0], max[1], max[2]],
                uv: arranged_uvs[3],
                normal,
                texture,
            },
        ],
        Direction::NegX => [
            Vertex {
                position: [min[0], min[1], min[2]],
                uv: arranged_uvs[0],
                normal,
                texture,
            },
            Vertex {
                position: [min[0], min[1], max[2]],
                uv: arranged_uvs[1],
                normal,
                texture,
            },
            Vertex {
                position: [min[0], max[1], max[2]],
                uv: arranged_uvs[2],
                normal,
                texture,
            },
            Vertex {
                position: [min[0], max[1], min[2]],
                uv: arranged_uvs[3],
                normal,
                texture,
            },
        ],
        Direction::PosY => [
            Vertex {
                position: [min[0], max[1], max[2]],
                uv: arranged_uvs[0],
                normal,
                texture,
            },
            Vertex {
                position: [max[0], max[1], max[2]],
                uv: arranged_uvs[1],
                normal,
                texture,
            },
            Vertex {
                position: [max[0], max[1], min[2]],
                uv: arranged_uvs[2],
                normal,
                texture,
            },
            Vertex {
                position: [min[0], max[1], min[2]],
                uv: arranged_uvs[3],
                normal,
                texture,
            },
        ],
        Direction::NegY => [
            Vertex {
                position: [min[0], min[1], min[2]],
                uv: arranged_uvs[0],
                normal,
                texture,
            },
            Vertex {
                position: [max[0], min[1], min[2]],
                uv: arranged_uvs[1],
                normal,
                texture,
            },
            Vertex {
                position: [max[0], min[1], max[2]],
                uv: arranged_uvs[2],
                normal,
                texture,
            },
            Vertex {
                position: [min[0], min[1], max[2]],
                uv: arranged_uvs[3],
                normal,
                texture,
            },
        ],
        Direction::PosZ => [
            Vertex {
                position: [min[0], min[1], max[2]],
                uv: arranged_uvs[0],
                normal,
                texture,
            },
            Vertex {
                position: [max[0], min[1], max[2]],
                uv: arranged_uvs[1],
                normal,
                texture,
            },
            Vertex {
                position: [max[0], max[1], max[2]],
                uv: arranged_uvs[2],
                normal,
                texture,
            },
            Vertex {
                position: [min[0], max[1], max[2]],
                uv: arranged_uvs[3],
                normal,
                texture,
            },
        ],
        Direction::NegZ => [
            Vertex {
                position: [max[0], min[1], min[2]],
                uv: arranged_uvs[0],
                normal,
                texture,
            },
            Vertex {
                position: [min[0], min[1], min[2]],
                uv: arranged_uvs[1],
                normal,
                texture,
            },
            Vertex {
                position: [min[0], max[1], min[2]],
                uv: arranged_uvs[2],
                normal,
                texture,
            },
            Vertex {
                position: [max[0], max[1], min[2]],
                uv: arranged_uvs[3],
                normal,
                texture,
            },
        ],
    }
}
