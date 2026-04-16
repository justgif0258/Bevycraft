use std::fmt::{Display, Formatter};
use bevycraft_core::prelude::AssetLocation;
use crate::prelude::{ArrayTexture, TextureId, Vertex};

#[derive(Debug, Clone, PartialEq)]
pub struct Quad {
    vertices: [Vertex; 4],
    facing  : Facing,
}

impl Quad {
    #[inline]
    pub fn new(
        from            : [f32; 3],
        to              : [f32; 3],
        uvs             : [f32; 4],
        scaling         : f32,
        facing          : Facing,
        texture         : &AssetLocation,
        array_texture   : &ArrayTexture
    ) -> Self {
        let texture = array_texture.get_texture_id(texture)
            .unwrap_or(TextureId(0));

        Self {
            vertices: Self::generate_vertex_array(from, to, uvs, scaling, facing, texture),
            facing,
        }
    }

    #[inline(always)]
    pub fn min(&self) -> Vertex {
        self.vertices[0].clone()
    }

    #[inline(always)]
    pub fn max(&self) -> Vertex {
        self.vertices[2].clone()
    }

    #[inline(always)]
    pub const fn facing(&self) -> Facing {
        self.facing
    }

    #[inline(always)]
    const fn generate_vertex_array(
        min: [f32; 3],
        max: [f32; 3],
        uvs: [f32; 4],
        scaling: f32,
        facing: Facing,
        texture: TextureId,
    ) -> [Vertex; 4] {
        let [min_x, min_y, min_z] = [min[0] * scaling, min[1] * scaling, min[2] * scaling];

        let [max_x, max_y, max_z] = [max[0] * scaling, max[1] * scaling, max[2] * scaling];

        let [u0, v0, u1, v1] = [uvs[0] * scaling, uvs[1] * scaling, uvs[2] * scaling, uvs[3] * scaling];

        let normal = facing.get_normal();

        match facing {
            Facing::PosX => [
                Vertex {
                    position: [max_x, min_y, max_z],
                    uv: [u0, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [max_x, min_y, min_z],
                    uv: [u1, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [max_x, max_y, min_z],
                    uv: [u1, v1],
                    normal,
                    texture,
                },
                Vertex {
                    position: [max_x, max_y, max_z],
                    uv: [u0, v1],
                    normal,
                    texture,
                },
            ],
            Facing::NegX => [
                Vertex {
                    position: [min_x, min_y, min_z],
                    uv: [u0, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [min_x, min_y, max_z],
                    uv: [u1, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [min_x, max_y, max_z],
                    uv: [u1, v1],
                    normal,
                    texture,
                },
                Vertex {
                    position: [min_x, max_y, min_z],
                    uv: [u0, v1],
                    normal,
                    texture,
                },
            ],
            Facing::PosY => [
                Vertex {
                    position: [min_x, max_y, max_z],
                    uv: [u0, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [max_x, max_y, max_z],
                    uv: [u1, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [max_x, max_y, min_z],
                    uv: [u1, v1],
                    normal,
                    texture,
                },
                Vertex {
                    position: [min_x, max_y, min_z],
                    uv: [u0, v1],
                    normal,
                    texture,
                },
            ],
            Facing::NegY => [
                Vertex {
                    position: [min_x, min_y, min_z],
                    uv: [u0, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [max_x, min_y, min_z],
                    uv: [u1, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [max_x, min_y, max_z],
                    uv: [u1, v1],
                    normal,
                    texture,
                },
                Vertex {
                    position: [min_x, min_y, max_z],
                    uv: [u0, v1],
                    normal,
                    texture,
                },
            ],
            Facing::PosZ => [
                Vertex {
                    position: [min_x, min_y, max_z],
                    uv: [u0, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [max_x, min_y, max_z],
                    uv: [u1, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [max_x, max_y, max_z],
                    uv: [u1, v1],
                    normal,
                    texture,
                },
                Vertex {
                    position: [min_x, max_y, max_z],
                    uv: [u0, v1],
                    normal,
                    texture,
                },
            ],
            Facing::NegZ => [
                Vertex {
                    position: [max_x, min_y, min_z],
                    uv: [u0, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [min_x, min_y, min_z],
                    uv: [u1, v0],
                    normal,
                    texture,
                },
                Vertex {
                    position: [min_x, max_y, min_z],
                    uv: [u1, v1],
                    normal,
                    texture,
                },
                Vertex {
                    position: [max_x, max_y, min_z],
                    uv: [u0, v1],
                    normal,
                    texture,
                },
            ],
        }
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
}

impl TryFrom<usize> for Facing {
    type Error = &'static str;

    #[inline(always)]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
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

impl TryFrom<u8> for Facing {
    type Error = &'static str;

    #[inline(always)]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
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

impl<'a> TryFrom<&'a str> for Facing {
    type Error = &'static str;

    #[inline(always)]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "east" => Ok(Facing::PosX),
            "west" => Ok(Facing::NegX),
            "up" => Ok(Facing::PosY),
            "down" => Ok(Facing::NegY),
            "south" => Ok(Facing::PosZ),
            "north" => Ok(Facing::NegZ),
            _ => Err("Invalid facing direction")
        }
    }
}

impl TryFrom<String> for Facing {
    type Error = &'static str;

    #[inline(always)]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "east" => Ok(Facing::PosX),
            "west" => Ok(Facing::NegX),
            "up" => Ok(Facing::PosY),
            "down" => Ok(Facing::NegY),
            "south" => Ok(Facing::PosZ),
            "north" => Ok(Facing::NegZ),
            _ => Err("Invalid facing direction")
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