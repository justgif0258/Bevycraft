use std::fmt::{Binary, Formatter};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::{info, Mesh};
use bevycraft_core::prelude::AssetLocation;
use crate::prelude::*;

const VERTEX_SCALING: f32 = 0.125f32;

const OCCLUSION_GRID: f32 = 8.0f32;

#[derive(Debug, Clone)]
pub struct BlockMesh {
    buckets     : [Vec<Quad>; 6],
    masks       : [OcclusionMask; 6],
    inner_faces : Vec<Quad>,
    render_flags: RenderFlags,
}

impl BlockMesh {
    #[inline]
    pub fn new(
        model           : RModel,
        manager         : &RModelManager,
        array_texture   : &ArrayTexture,
        render_flags    : RenderFlags
    ) -> Result<Self, Vec<String>> {
        let mut errors: Vec<String> = Vec::new();

        let mut buckets: [Vec<Quad>; 6] = Default::default();
        let mut inner_faces : Vec<Quad> = Vec::new();

        let geometry = if let Some(elements) = &model.elements {
            elements
        } else if let Some (elements) = manager.try_load_parent(&model) {
            elements
        } else {
            errors.push("Failed to bake block mesh elements".to_string());

            return Err(errors);
        };

        if let Some(textures) = model.textures {
            geometry.iter()
                .for_each(|element| {
                    element.faces
                        .iter()
                        .for_each(|(facing, face)| {
                            match Facing::try_from(facing.as_str()) {
                                Err(e) => errors.push(e.to_string()),
                                Ok(facing) => {
                                    let texture = match face.texture.strip_prefix('#') {
                                        None => AssetLocation::try_parse(&face.texture).ok(),
                                        Some(key) => textures.get(key).cloned(),
                                    };

                                    if let Some(texture) = &texture {
                                        let quad = Quad::new(
                                            element.from,
                                            element.to,
                                            face.uv,
                                            VERTEX_SCALING,
                                            facing,
                                            face.tintable,
                                            texture,
                                            array_texture,
                                        );

                                        if let Some(cullface) = &face.cullface
                                            && let Ok(facing_cull) = Facing::try_from(cullface.as_str())
                                        {
                                            buckets[facing_cull as usize].push(quad);
                                        } else {
                                            inner_faces.push(quad);
                                        }
                                    } else {
                                        errors.push(format!("Failed to load texture {} for {} face", &face.texture, facing))
                                    }
                                }
                            }
                        })
                })
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let mut masks = [OcclusionMask(0); 6];

        for facing in 0..6 {
            let mut mask = OcclusionMask(0);

            buckets[facing]
                .iter()
                .for_each(|quad| {
                    let face = Facing::try_from(facing).unwrap();

                    let [min_x, min_y, min_z] = quad.min();
                    let [max_x, max_y, max_z] = quad.max();

                    match face {
                        Facing::PosX | Facing::NegX => {
                            mask.fill_bits(
                                [min_z, min_y],
                                [max_z, max_y],
                                true
                            );
                        }
                        Facing::PosY | Facing::NegY => {
                            mask.fill_bits(
                                [min_x, min_z],
                                [max_x, max_z],
                                true
                            );
                        }
                        Facing::PosZ | Facing::NegZ => {
                            mask.fill_bits(
                                [min_x, min_y],
                                [max_x, max_y],
                                true
                            );
                        }
                    }

                    masks[facing] = mask;
                })
        }

        Ok(Self {
            buckets,
            inner_faces,
            masks,
            render_flags,
        })
    }

    #[inline(always)]
    pub fn get_quads_at(&self, facing: Facing) -> &[Quad] {
        self.buckets[facing as usize]
            .as_slice()
    }

    #[inline(always)]
    pub fn get_inner_quads(&self) -> &[Quad] {
        &self.inner_faces
    }

    #[inline(always)]
    pub fn occlusion_mask(&self, facing: Facing) -> OcclusionMask {
        self.masks[facing as usize]
    }

    #[inline(always)]
    pub fn is_occluded_at(&self, facing: Facing, mask: OcclusionMask) -> bool {
        self.masks[facing as usize].is_occluded(mask)
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &Quad> {
        self.buckets.iter()
            .flatten()
            .chain(self.inner_faces.iter())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct OcclusionMask(u64);

impl OcclusionMask {
    #[inline(always)]
    pub const fn is_occluded(&self, other: Self) -> bool {
        (other.0 & self.0) == self.0
    }

    #[inline(always)]
    fn fill_bits(&mut self, from: [f32; 2], to: [f32; 2], value: bool) {
        let i_from_u = (from[0].min(to[0]) * OCCLUSION_GRID).ceil() as u32;

        let i_from_v = (from[1].min(to[1]) * OCCLUSION_GRID).ceil() as u32;

        let i_to_u = (to[0].max(from[0]) * OCCLUSION_GRID).floor() as u32;

        let i_to_v = (to[1].max(from[1]) * OCCLUSION_GRID).floor() as u32;

        for u in i_from_u..i_to_u {
            for v in i_from_v..i_to_v {
                let pos = Self::map_to_bit_index(u, v);

                self.0 &= !(1u64 << pos);
                self.0 |= (value as u64) << pos;
            }
        }
    }

    #[inline(always)]
    const fn map_to_bit_index(u: u32, v: u32) -> u32 {
        (v * 8) + u
    }
}

impl Binary for OcclusionMask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:064b}", self.0)
    }
}

bitflags::bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct RenderFlags: u8 {
        const OPAQUE            = 0x1;
        const TRANSLUCENT       = 0x2;
        const EMISSIVE          = 1 << 2;
        const OCCLUDABLE        = 1 << 3;
        const GREEDY_MESHABLE   = 1 << 4;
    }
}