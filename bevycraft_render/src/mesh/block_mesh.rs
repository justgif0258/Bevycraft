use std::fmt::{Binary, Formatter};

use bevy::{
    ecs::resource::Resource,
    platform::{collections::HashMap, hash::NoOpHash},
};
use bevycraft_core::prelude::BlockType;
use frozen_collections::FzHashMap;

use crate::prelude::{ArrayTexture, Facing, Quad, RModel};

const VERTEX_SCALING: f32 = 0.125f32;

const OCCLUSION_GRID: f32 = 8.0f32;

#[derive(Resource)]
pub struct BlockMeshCache {
    meshes: FzHashMap<BlockType, BlockMesh, NoOpHash>,
}

impl BlockMeshCache {
    #[inline]
    pub const fn builder() -> MeshCacheBuilder {
        MeshCacheBuilder {
            entries: HashMap::with_hasher(NoOpHash),
        }
    }

    #[inline(always)]
    pub fn get_mesh(&self, block_type: BlockType) -> Option<&BlockMesh> {
        if block_type.is_air() {
            return None;
        }

        self.meshes.get(&block_type)
    }

    #[inline(always)]
    pub fn get_occlusion_mask(
        &self,
        block_type: BlockType,
        facing: Facing,
    ) -> Option<OcclusionMask> {
        if block_type.is_air() {
            return None;
        }

        let mesh = self.get_mesh(block_type)?;

        Some(mesh.occlusion_mask(facing))
    }
}

#[derive(Default)]
pub struct MeshCacheBuilder {
    entries: HashMap<BlockType, BlockMesh, NoOpHash>,
}

impl MeshCacheBuilder {
    #[inline]
    pub fn add_mesh(&mut self, mesh: BlockMesh, block_id: BlockType) {
        self.entries.insert(block_id, mesh);
    }

    #[inline]
    pub fn bake_and_add_mesh(
        &mut self,
        manager: &RModelManager,
        array_texture: &ArrayTexture,
        model: &RModel,
        block_id: BlockType,
    ) -> Result<(), Vec<String>> {
        let mesh = BlockMesh::new(model, &manager, &array_texture);

        match mesh {
            Ok(mesh) => {
                self.entries.insert(block_id, mesh);

                Ok(())
            }
            Err(errors) => Err(errors),
        }
    }

    #[inline]
    pub fn build(self) -> BlockMeshCache {
        let meshes: FzHashMap<BlockType, BlockMesh, NoOpHash> =
            FzHashMap::from_iter(self.entries.into_iter());

        BlockMeshCache { meshes }
    }
}

/// # Block Mesh
/// Efficiently baked and solved mesh for a given block, ready to be rendered with binary occlusion masking.
#[derive(Debug, Clone)]
pub struct BlockMesh {
    buckets: [Box<[Quad]>; 6],
    masks: [OcclusionMask; 6],
    inner_faces: Box<[Quad]>,
}

impl BlockMesh {
    #[inline]
    pub fn new(
        model: &RModel,
        manager: &RModelManager,
        textures: &ArrayTexture,
    ) -> Result<Self, Vec<String>> {
        let mut errors: Vec<String> = Vec::new();

        let mut buckets: [Vec<Quad>; 6] = Default::default();
        let mut inner_faces: Vec<Quad> = Vec::new();

        let geometry = if let Some(elements) = &model.elements {
            elements
        } else if let Some(elements) = manager.try_load_parent(&model) {
            elements
        } else {
            errors.push("Failed to bake block mesh elements".to_string());

            return Err(errors);
        };

        if let Some(textures_names) = &model.textures {
            geometry.iter().for_each(|element| {
                element.faces.iter().for_each(|(direction, face)| {
                    /*
                    match Facing::try_from(direction.as_str()) {
                        Err(e) => errors.push(e.to_string()),
                        Ok(facing) => {
                            let texture = match face.texture.strip_prefix('#') {
                                None => AssetLocation::try_parse(&face.texture).ok(),
                                Some(key) => textures_names.get(key).cloned(),
                            };

                            if let Some(texture_location) = &texture
                                && let Some(texture) = textures.get_texture_id(texture_location)
                            {
                                let scaled_min = [
                                    element.from[0] * VERTEX_SCALING,
                                    element.from[1] * VERTEX_SCALING,
                                    element.from[2] * VERTEX_SCALING
                                ];

                                let scaled_max = [
                                    element.to[0] * VERTEX_SCALING,
                                    element.to[1] * VERTEX_SCALING,
                                    element.to[2] * VERTEX_SCALING
                                ];

                                let scaled_uv = [
                                    face.uv[0] * VERTEX_SCALING,
                                    face.uv[1] * VERTEX_SCALING,
                                    face.uv[2] * VERTEX_SCALING,
                                    face.uv[3] * VERTEX_SCALING
                                ];

                                let mut quad = Quad::new(
                                    scaled_min,
                                    scaled_max,
                                    scaled_uv,
                                    texture,
                                    face.render_mode,
                                    face.tintable,
                                    facing,
                                );

                                if let Some(rot) = &element.rotation {
                                    let origin = Vec3::from(rot.origin) * VERTEX_SCALING;

                                    quad.rotate(
                                        origin,
                                        rot.x,
                                        rot.y,
                                        rot.z,
                                    );
                                }

                                if let Some(cullface) = &face.cullface {
                                    buckets[*cullface as usize].push(quad);
                                } else {
                                    inner_faces.push(quad);
                                }
                            } else {
                                errors.push(format!("Failed to load texture {} for {} face", &face.texture, facing))
                            }
                        }
                    }
                     */
                })
            })
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let mut masks = [OcclusionMask(0); 6];

        for facing in 0..6 {
            let mut mask = OcclusionMask(0);

            buckets[facing].iter().for_each(|quad| {
                let face = Facing::try_from(facing).unwrap();

                let [min_x, min_y, min_z] = quad.min();
                let [max_x, max_y, max_z] = quad.max();

                match face {
                    Facing::PosX | Facing::NegX => {
                        mask.fill_bits([min_z, min_y], [max_z, max_y], true);
                    }
                    Facing::PosY | Facing::NegY => {
                        mask.fill_bits([min_x, min_z], [max_x, max_z], true);
                    }
                    Facing::PosZ | Facing::NegZ => {
                        mask.fill_bits([min_x, min_y], [max_x, max_y], true);
                    }
                }

                masks[facing] = mask;
            })
        }

        let buckets: [Box<[Quad]>; 6] = buckets.map(Vec::into_boxed_slice);

        let inner_faces: Box<[Quad]> = inner_faces.into_boxed_slice();

        Ok(Self {
            buckets,
            inner_faces,
            masks,
        })
    }

    #[inline(always)]
    pub const fn get_occlusion_quads_at(&self, facing: Facing) -> &[Quad] {
        &self.buckets[facing as usize]
    }

    #[inline(always)]
    pub const fn get_inner_quads(&self) -> &[Quad] {
        &self.inner_faces
    }

    #[inline(always)]
    pub const fn occlusion_mask(&self, facing: Facing) -> OcclusionMask {
        self.masks[facing as usize]
    }

    #[inline(always)]
    pub const fn is_occluded_at(&self, facing: Facing, mask: OcclusionMask) -> bool {
        self.masks[facing as usize].is_occluded(mask)
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &Quad> {
        self.buckets.iter().flatten().chain(self.inner_faces.iter())
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
        const CUTOUT            = 1 << 0;
        const TRANSLUCENT       = 1 << 1;
        const EMISSIVE          = 1 << 2;
        const OCCLUDABLE        = 1 << 3;
        const GREEDY_MESHABLE   = 1 << 4;
    }
}

impl RenderFlags {
    #[inline(always)]
    pub const fn is_opaque(&self) -> bool {
        !self.contains(RenderFlags::CUTOUT) && !self.contains(RenderFlags::TRANSLUCENT)
    }

    #[inline(always)]
    pub const fn is_cutout(&self) -> bool {
        self.contains(RenderFlags::CUTOUT)
    }

    #[inline(always)]
    pub const fn is_translucent(&self) -> bool {
        self.contains(RenderFlags::TRANSLUCENT)
    }
}
