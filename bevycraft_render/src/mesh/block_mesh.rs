use bevycraft_core::prelude::AssetLocation;
use crate::prelude::{ArrayTexture, Facing, Quad, RModel, RModelManager};

#[derive(Debug, Clone)]
pub struct BlockMesh {
    buckets     : [Vec<Quad>; 6],
    masks       : [OcclusionMask; 6],
    inner_faces : Vec<Quad>,
    flags       : RenderFlags,
}

impl BlockMesh {
    pub fn new(
        model           : RModel,
        manager         : &RModelManager,
        array_texture   : &ArrayTexture
    ) -> Option<Self> {
        let mut buckets: [Vec<Quad>; 6] = Default::default();
        let mut inner_faces : Vec<Quad> = Vec::new();

        let geometry = if let Some(elements) = &model.elements {
            elements
        } else if let Some (elements) = manager.try_load_parent(&model) {
            elements
        } else {
            return None;
        };

        let texture_map = model.textures.unwrap();

        geometry.iter()
            .for_each(|geo| {
                geo.faces
                    .iter()
                    .for_each(|(facing, face)| {
                        let facing = Facing::try_from(facing.as_str()).unwrap();

                        let texture = match face.texture.starts_with('#') {
                            true => texture_map.get(&face.texture).unwrap(),
                            false => &AssetLocation::parse(&face.texture)
                        };

                        if face.cullface.is_some() {
                            buckets[facing as usize].push(
                                Quad::new(
                                    geo.from,
                                    geo.to,
                                    face.uv,
                                    facing,
                                    texture,
                                    array_texture
                                )
                            );
                        } else {
                            inner_faces.push(
                                Quad::new(
                                    geo.from,
                                    geo.to,
                                    face.uv,
                                    facing,
                                    texture,
                                    array_texture
                                )
                            );
                        }
                    });
            });

        todo!()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct OcclusionMask(u64);

impl OcclusionMask {
    #[inline(always)]
    fn fill_bits(&mut self, from: [i32; 2], to: [i32; 2], value: bool) {
        let width = to[0] - from[0];
        let height = to[1] - from[1];

        let area = width * height;

        for x in from[0]..to[0] {
            for y in from[1]..to[1] {
                let pos = Self::map_to_bit_index(x, y);

                self.0 &= !(0b1 << pos);
                self.0 |= (value as u64) << pos;
            }
        }
    }

    #[inline(always)]
    const fn is_contained(&self, other: Self) -> bool {
        self.0 & other.0 == self.0
    }

    #[inline(always)]
    const fn map_to_bit_index(x: i32, y: i32) -> u64 {
        (x + (y * 8)) as u64
    }
}

bitflags::bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    struct RenderFlags: u8 {
        const OPAQUE            = 0x1;
        const TRANSLUCENT       = 0x2;
        const EMISSIVE          = 1 << 2;
        const OCCLUDABLE        = 1 << 3;
        const GREEDY_MESHABLE   = 1 << 4;
    }
}