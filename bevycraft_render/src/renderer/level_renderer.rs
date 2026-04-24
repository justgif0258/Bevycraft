use std::sync::Arc;
use bevy::math::IVec3;
use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::{Assets, Commands, Component, Entity, Mesh, Mesh3d, MeshMaterial3d, Resource, Transform};
use bevycraft_world::prelude::{ChunkPos, ChunkState, Level, SECTION_SIZE};
use crate::prelude::{ArrayTexture, BlockMeshCache, ChunkMesh, Facing, MeshBuffer, RenderMode};

#[derive(Resource)]
pub struct LevelRenderer {
    active_meshes: HashMap<ChunkPos, Entity, NoOpHash>,
    mesh_cache: Arc<BlockMeshCache>,
    materials: Arc<ArrayTexture>
}

impl LevelRenderer {
    #[inline]
    pub fn new(
        mesh_cache: Arc<BlockMeshCache>,
        materials: Arc<ArrayTexture>,
    ) -> Self {
        Self {
            active_meshes: HashMap::with_hasher(NoOpHash),
            mesh_cache,
            materials
        }
    }

    #[inline]
    pub fn render_chunk(
        &mut self,
        commands:   &mut Commands,
        meshes:     &mut Assets<Mesh>,
        level:      &Level,
        chunk_pos:  ChunkPos,
    ) {
        if let Some(chunk) = level.get_chunk_state(&chunk_pos) {
            if let ChunkState::Loaded(loaded) = chunk {
                let mut builder = MeshBuffer::new();

                let world_pos = chunk_pos.into_world_pos().as_ivec3();

                loaded.sections
                    .iter()
                    .for_each(|(&index, section)| {
                        let world_height = index.into_world_height();

                        for x in 0..SECTION_SIZE {
                            for y in 0..SECTION_SIZE {
                                for z in 0..SECTION_SIZE {
                                    let b_type = section.get([x, y, z]);

                                    if b_type.is_air() {
                                        continue;
                                    }

                                    let cached = self.mesh_cache.get_mesh(b_type)
                                        .unwrap();

                                    for f in 0..6u8 {
                                        let facing = Facing::try_from(f).unwrap();

                                        let neighbor_pos = match facing {
                                            Facing::PosX => IVec3::new(world_pos.x + 1, world_pos.y, world_pos.z),
                                            Facing::NegX => IVec3::new(world_pos.x - 1, world_pos.y, world_pos.z),
                                            Facing::PosY => IVec3::new(world_pos.x, world_pos.y + 1, world_pos.z),
                                            Facing::NegY => IVec3::new(world_pos.x, world_pos.y - 1, world_pos.z),
                                            Facing::PosZ => IVec3::new(world_pos.x, world_pos.y, world_pos.z + 1),
                                            Facing::NegZ => IVec3::new(world_pos.x, world_pos.y, world_pos.z - 1),
                                        };

                                        if let Some(neighbor) = level.get_at(neighbor_pos)
                                            && let Some(mask) = self.mesh_cache.get_occlusion_mask(neighbor, !facing)
                                        {
                                            if cached.is_occluded_at(facing, mask) {
                                                continue;
                                            }

                                            builder.push_quads_with_offset(
                                                cached.get_occlusion_quads_at(facing),
                                                Some([0.2, 0.8, 0.2, 1.0]),
                                                [x as f32, (world_height + y) as f32, z as f32],
                                            )
                                        }
                                    }

                                    builder.push_quads_with_offset(
                                        cached.get_inner_quads(),
                                        Some([0.2, 0.8, 0.2, 1.0]),
                                        [x as f32, (world_height + y) as f32, z as f32],
                                    )
                                }
                            }
                        }
                    });

                let mesh = builder.mesh();

                let entity = commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    Transform::from_translation(chunk_pos.into_world_pos()),
                )).id();

                if !self.active_meshes.contains_key(&chunk_pos) {
                    self.active_meshes.insert(chunk_pos, entity);
                }
            }
        }
    }
}

#[derive(Component)]
pub struct ChunkMeshEntity;