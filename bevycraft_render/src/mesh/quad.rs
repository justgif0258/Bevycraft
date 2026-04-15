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
        facing          : Facing,
        texture         : &AssetLocation,
        array_texture   : &ArrayTexture
    ) -> Self {
        let texture = array_texture.get_texture_id(texture)
            .unwrap_or(TextureId(0));

        Self {
            vertices: Self::generate_vertex_array(from, to, facing, uvs, texture),
            facing,
        }
    }

    #[inline(always)]
    const fn generate_vertex_array(
        min: [f32; 3],
        max: [f32; 3],
        facing: Facing,
        uvs: [f32; 4],
        texture: TextureId,
    ) -> [Vertex; 4] {
        let [min_x, min_y, min_z] = min;
        let [max_x, max_y, max_z] = max;

        let [u0, v0, u1, v1] = uvs;

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