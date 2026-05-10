use bevy::{asset::Asset, reflect::TypePath};
use bevycraft_core::prelude::AssetLocation;

use crate::prelude::*;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct BlockModel {
    outer_quads: [Box<[Quad]>; 6],
    inner_quads: Box<[Quad]>,
    masks: [OcclusionMask; 6],
}

impl Model for BlockModel {
    type Raw = RModel;

    fn resolve(raw: Self::Raw, textures: &TextureRegistry) -> Self {
        let mut outer_quads = [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ];
        let mut inner_quads = Vec::new();
        let mut masks = [OcclusionMask::EMPTY; 6];

        raw.elements.iter().for_each(|element| {
            let [x0, y0, z0] = element.from;
            let [x1, y1, z1] = element.to;

            element.faces.iter().for_each(|(direction, face)| {
                let texture = match face.texture.strip_prefix('#') {
                    Some(t) => {
                        if let Some(loc) = raw.textures.get(t) {
                            textures.get_or_insert(loc)
                        } else {
                            NULL_TEXTURE_ID
                        }
                    }
                    None => textures.get_or_insert(&AssetLocation::parse(&face.texture)),
                };

                let (from, to) = match direction {
                    Direction::PosX | Direction::NegX => ([z0, y0], [z1, y1]),
                    Direction::PosY | Direction::NegY => ([x0, z0], [x1, z1]),
                    Direction::PosZ | Direction::NegZ => ([x0, y0], [x1, y1]),
                };

                let depth = match direction {
                    Direction::PosX => x1,
                    Direction::NegX => x0,
                    Direction::PosY => y1,
                    Direction::NegY => y0,
                    Direction::PosZ => z1,
                    Direction::NegZ => z0,
                };

                match face.cullface {
                    Some(cullface) => {
                        let quad = Quad::with_occlusion_mask(
                            *direction,
                            from,
                            to,
                            depth,
                            face.uv,
                            texture,
                            face.render_mode,
                            face.tintable,
                        );

                        masks[cullface as usize].merge_assign(quad.mask());

                        outer_quads[cullface as usize].push(quad);
                    }
                    None => {
                        let quad = Quad::new(
                            *direction,
                            from,
                            to,
                            depth,
                            face.uv,
                            texture,
                            face.render_mode,
                            face.tintable,
                        );
                        inner_quads.push(quad);
                    }
                }
            });
        });

        let outer_quads = outer_quads.map(|v| v.into_boxed_slice());
        let inner_quads = inner_quads.into_boxed_slice();

        Self {
            outer_quads,
            inner_quads,
            masks,
        }
    }
}

impl BlockModel {
    #[inline]
    pub fn iter_outer_quads_at(&self, dir: Direction) -> std::slice::Iter<'_, Quad> {
        self.outer_quads[dir as usize].iter()
    }

    #[inline]
    pub fn iter_inner_quads(&self) -> std::slice::Iter<'_, Quad> {
        self.inner_quads.iter()
    }

    #[inline]
    pub const fn mask(&self, dir: Direction) -> OcclusionMask {
        self.masks[dir as usize]
    }
}
