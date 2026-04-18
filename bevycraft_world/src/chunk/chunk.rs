use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use bevycraft_render::prelude::{BlockMeshManager, MeshBuffer};
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

        let normalized = UVec3::new(
            position.x.rem_euclid(SECTION_SIZE as i32) as u32,
            position.y.rem_euclid(SECTION_SIZE as i32) as u32,
            position.z.rem_euclid(SECTION_SIZE as i32) as u32,
        );

        let y_idx = position.y.div_euclid(SECTION_SIZE as i32) as u64;

        if let Some(section) = self.sections.get_mut(&y_idx) {
            section.set(normalized, global_index);

            return;
        }

        let mut new_section = PalettedSection::new();

        new_section.set(normalized, global_index);

        self.sections.insert(y_idx, new_section);
    }

    #[inline(always)]
    pub fn get_at(
        &self,
        position: impl Into<IVec3>,
    ) -> Option<u32> {
        let position = position.into();

        let normalized = UVec3::new(
            position.x.rem_euclid(SECTION_SIZE as i32) as u32,
            position.y.rem_euclid(SECTION_SIZE as i32) as u32,
            position.z.rem_euclid(SECTION_SIZE as i32) as u32,
        );

        let y_idx = position.y.div_euclid(SECTION_SIZE as i32) as u64;

        if let Some(section) = self.sections.get(&y_idx) {
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
                    self.pos.x * SECTION_SIZE as i32,
                    *y_idx as i32 * SECTION_SIZE as i32,
                    self.pos.y * SECTION_SIZE as i32,
                );

                for x in 0..SECTION_SIZE {
                    for y in 0..SECTION_SIZE {
                        for z in 0..SECTION_SIZE {
                            if let Some(idx) = section.get([x, y, z])
                                && let Some(mesh) = manager.get_mesh(idx)
                            {
                                let current = world_pos + UVec3::new(x, y, z).as_ivec3();

                                buffer.push_mesh(mesh, Some([0.2, 0.8, 0.2, 1.0]), current.as_vec3().into())
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