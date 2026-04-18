use std::ops::Add;
use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use bevycraft_render::prelude::{BlockMeshManager, Facing, MeshBuffer};
use crate::prelude::*;

pub struct Chunk {
    pub pos: IVec2,

    pub sections: HashMap<u64, PalettedSection, NoOpHash>,

    pub dirty: bool,
}

impl Chunk {
    #[inline(always)]
    pub fn new(position: impl Into<IVec2>) -> Self {
        Self {
            pos: position.into(),
            sections: HashMap::with_hasher(NoOpHash),
            dirty: false,
        }
    }

    #[inline(always)]
    pub fn generate_using(
        position: impl Into<IVec2>,
        blocks: &BlockRecord,
        generator: &dyn WorldGenerator
    ) -> Self {
        let mut chunk = Self::new(position);

        generator.generate_base_terrain(
            &mut chunk,
            blocks,
        );

        generator.carve_terrain(
            &mut chunk,
            blocks,
        );

        generator.generate_features(
            &mut chunk,
            blocks,
        );

        chunk
    }

    #[inline(always)]
    pub fn set_at(
        &mut self,
        position: impl Into<IVec3>,
        global_index: u32
    ) {
        let position = position.into();

        let normalized = position.rem_euclid(IVec3::splat(SECTION_SIZE));

        let y_idx = position.y.div_euclid(SECTION_SIZE) as u64;

        if let Some(section) = self.sections.get_mut(&y_idx) {
            section.set(normalized, global_index);

            return;
        }

        let mut new_section = PalettedSection::new();

        new_section.set(normalized, global_index);

        self.sections.insert(y_idx, new_section);
    }

    #[inline(always)]
    pub fn remove_at(&mut self, position: impl Into<IVec3>) {
        let position = position.into();

        let y_idx = position.y.div_euclid(SECTION_SIZE) as u64;

        if let Some(section) = self.sections.get_mut(&y_idx) {
            let normalized = position.rem_euclid(IVec3::splat(SECTION_SIZE));

            section.remove(normalized);
        }
    }

    #[inline(always)]
    pub fn get_at(
        &self,
        position: impl Into<IVec3>,
    ) -> Option<u32> {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() {
            return None;
        }

        if position.x >= SECTION_SIZE || position.z >= SECTION_SIZE {
            return None;
        }

        let y_idx = position.y.div_euclid(SECTION_SIZE) as u64;

        if let Some(section) = self.sections.get(&y_idx) {
            let normalized = IVec3::new(
                position.x,
                position.y.rem_euclid(SECTION_SIZE),
                position.z,
            );

            return section.get(normalized);
        }

        None
    }

    #[inline(always)]
    pub fn build_chunk_mesh(
        &self,
        manager: &BlockMeshManager,
    ) -> Mesh {
        let mut buffer = MeshBuffer::with_expected_capacity(256);

        self.sections.iter()
            .for_each(|(y_idx, section)| {
                let world_pos = IVec3::new(
                    self.pos.x * SECTION_SIZE,
                    *y_idx as i32 * SECTION_SIZE,
                    self.pos.y * SECTION_SIZE,
                );

                for x in 0..SECTION_SIZE {
                    for y in 0..SECTION_SIZE {
                        for z in 0..SECTION_SIZE {
                            let local = IVec3::new(x, y, z);

                            let actual_y = world_pos.y + local.y;

                            if let Some(idx) = section.get(local)
                                && let Some(mesh) = manager.get_mesh(idx)
                            {
                                let current = world_pos + local;

                                for facing in 0..6usize {
                                    let facing = Facing::try_from(facing).unwrap();

                                    let neighbor = match facing {
                                        Facing::PosX => IVec3::new(local.x + 1, actual_y, local.z),
                                        Facing::NegX => IVec3::new(local.x - 1, actual_y, local.z),
                                        Facing::PosY => IVec3::new(local.x, actual_y + 1, local.z),
                                        Facing::NegY => IVec3::new(local.x, actual_y - 1, local.z),
                                        Facing::PosZ => IVec3::new(local.x, actual_y, local.z + 1),
                                        Facing::NegZ => IVec3::new(local.x, actual_y, local.z - 1),
                                    };

                                    if let Some(block_at) = self.get_at(neighbor) {
                                        let mask = manager.get_occlusion_mask(block_at, !facing)
                                            .unwrap();

                                        if mesh.is_occluded_at(facing, mask) {
                                            continue;
                                        }

                                        buffer.push_quads(
                                            mesh.get_quads_at(facing),
                                            Some([0.2, 0.8, 0.2, 1.0]),
                                            current.as_vec3().into()
                                        )
                                    } else {
                                        buffer.push_quads(
                                            mesh.get_quads_at(facing),
                                            Some([0.2, 0.8, 0.2, 1.0]),
                                            current.as_vec3().into()
                                        )
                                    }
                                }

                                buffer.push_quads(
                                    mesh.get_inner_quads(),
                                    Some([0.2, 0.8, 0.2, 1.0]),
                                    current.as_vec3().into()
                                )
                            }
                        }
                    }
                }
            });

        buffer.render()
    }

    #[inline(always)]
    pub const fn world_pos(&self) -> IVec2 {
        IVec2::new(
            self.pos.x * (SECTION_SIZE as i32),
            self.pos.y * (SECTION_SIZE as i32),
        )
    }
}