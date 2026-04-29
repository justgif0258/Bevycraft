use std::fmt::{Display, Formatter};
use std::ops::Not;
use bevy::math::EulerRot;
use bevy::prelude::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use crate::prelude::{TextureId, Vertex};

pub const NEUTRAL_TINT: [f32; 4] = [1.0; 4];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    vertices    : [Vertex; 4],
    render_mode : RenderMode,
    tintable    : bool,
}

impl Quad {
    #[inline]
    pub fn new(
        min     : [f32; 3],
        max     : [f32; 3],
        uv      : [f32; 4],
        texture : TextureId,
        mode    : RenderMode,
        tintable: bool,
        facing  : Facing,
    ) -> Self {
        Self {
            vertices: build_vertex_array(min, max, uv, texture, facing),
            render_mode: mode,
            tintable,
        }
    }

    #[inline]
    pub fn rotate(
        &mut self,
        origin: Vec3,
        x:      f32,
        y:      f32,
        z:      f32
    ) {
        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            x.to_radians(),
            y.to_radians(),
            z.to_radians()
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

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Facing {
    PosX = 0, NegX = 1,
    PosY = 2, NegY = 3,
    PosZ = 4, NegZ = 5,
}

impl Facing {

    #[inline(always)]
    pub const fn get_normal(self) -> [f32; 3] {
        match self {
            Facing::PosX => [1.0, 0.0, 0.0],
            Facing::NegX => [-1.0, 0.0, 0.0],
            Facing::PosY => [0.0, 1.0, 0.0],
            Facing::NegY => [0.0, -1.0, 0.0],
            Facing::PosZ => [0.0, 0.0, 1.0],
            Facing::NegZ => [0.0, 0.0, -1.0],
        }
    }

    #[inline(always)]
    fn from_str(str: impl AsRef<str>) -> Result<Self, String> {
        match str.as_ref() {
            "east" => Ok(Facing::PosX),
            "west" => Ok(Facing::NegX),
            "up" => Ok(Facing::PosY),
            "down" => Ok(Facing::NegY),
            "south" => Ok(Facing::PosZ),
            "north" => Ok(Facing::NegZ),
            _ => Err(format!("Unknown face direction: {}", str.as_ref())),
        }
    }

    #[inline(always)]
    const fn from_value(value: u8) -> Result<Self, &'static str> {
        match value {
            0 => Ok(Facing::PosX),
            1 => Ok(Facing::NegX),
            2 => Ok(Facing::PosY),
            3 => Ok(Facing::NegY),
            4 => Ok(Facing::PosZ),
            5 => Ok(Facing::NegZ),
            _ => Err("Invalid facing value")
        }
    }
}

impl Display for Facing {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Facing::PosX => f.write_str("east"),
            Facing::NegX => f.write_str("west"),
            Facing::PosY => f.write_str("up"),
            Facing::NegY => f.write_str("down"),
            Facing::PosZ => f.write_str("south"),
            Facing::NegZ => f.write_str("north"),
        }
    }
}

impl Not for Facing {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        match self {
            Facing::PosX => Facing::NegX,
            Facing::NegX => Facing::PosX,
            Facing::PosY => Facing::NegY,
            Facing::NegY => Facing::PosY,
            Facing::PosZ => Facing::NegZ,
            Facing::NegZ => Facing::PosZ,
        }
    }
}

impl TryFrom<usize> for Facing {
    type Error = &'static str;

    #[inline(always)]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::from_value(value as u8)
    }
}

impl TryFrom<u8> for Facing {
    type Error = &'static str;

    #[inline(always)]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_value(value)
    }
}

impl<'a> TryFrom<&'a str> for Facing {
    type Error = String;

    #[inline(always)]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<String> for Facing {
    type Error = String;

    #[inline(always)]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RenderMode {
    Opaque = 0,
    Cutout = 1,
    Translucent = 2,
}

impl RenderMode {
    #[inline(always)]
    pub fn from_str(str: impl AsRef<str>) -> Result<Self, String> {
        match str.as_ref() {
            "opaque" => Ok(RenderMode::Opaque),
            "cutout" => Ok(RenderMode::Cutout),
            "translucent" => Ok(RenderMode::Translucent),
            _ => Err(format!("Unknown render mode: {}", str.as_ref())),
        }
    }
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
    facing: Facing,
) -> [Vertex; 4] {
    let arranged_uvs = [
        [uv[0], uv[3]],
        [uv[2], uv[3]],
        [uv[2], uv[1]],
        [uv[0], uv[1]],
    ];

    let normal = facing.get_normal();

    match facing {
        Facing::PosX => [
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
        Facing::NegX => [
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
        Facing::PosY => [
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
        Facing::NegY => [
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
        Facing::PosZ => [
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
        Facing::NegZ => [
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