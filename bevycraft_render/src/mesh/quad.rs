use crate::mesh::occlusion_mask::OcclusionMask;
use crate::prelude::TextureId;
use bevy::math::EulerRot;
use bevy::prelude::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    positions: [[f32; 3]; 4],
    uvs: [[f32; 2]; 4],
    normal: [f32; 3],
    mask: OcclusionMask,
    texture: TextureId,
    render_mode: RenderMode,
    tintable: bool,
}

impl Quad {
    #[inline]
    pub fn new(
        dir: Direction,
        from: [f32; 2],
        to: [f32; 2],
        depth: f32,
        uv: [f32; 4],
        texture: TextureId,
        render_mode: RenderMode,
        tintable: bool,
    ) -> Self {
        let [[x0, y0], [x1, y1]] = [from, to];
        let [u0, v0, u1, v1] = uv;

        let mut corners = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];

        let mut uvs = [[u0, v1], [u1, v1], [u1, v0], [u0, v0]];

        if matches!(dir, Direction::NegX | Direction::NegY | Direction::NegZ) {
            corners.swap(1, 3);
            uvs.swap(1, 3);
        }

        let positions = corners.map(|[x, y]| match dir {
            Direction::PosX | Direction::NegX => [depth, y, x],
            Direction::PosY | Direction::NegY => [x, depth, y],
            Direction::PosZ | Direction::NegZ => [x, y, depth],
        });

        let normal = dir.get_normal();

        Self {
            positions,
            uvs,
            normal,
            mask: OcclusionMask::EMPTY,
            texture,
            render_mode,
            tintable,
        }
    }

    #[inline]
    pub fn with_occlusion_mask(
        dir: Direction,
        from: [f32; 2],
        to: [f32; 2],
        depth: f32,
        uv: [f32; 4],
        texture: TextureId,
        render_mode: RenderMode,
        tintable: bool,
    ) -> Self {
        let [[x0, y0], [x1, y1]] = [from, to];
        let [u0, v0, u1, v1] = uv;

        let mut corners = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];

        let mut uvs = [[u0, v1], [u1, v1], [u1, v0], [u0, v0]];

        if matches!(dir, Direction::NegX | Direction::NegY | Direction::NegZ) {
            corners.swap(1, 3);
            uvs.swap(1, 3);
        }

        let positions = corners.map(|[x, y]| match dir {
            Direction::PosX | Direction::NegX => [depth, y, x],
            Direction::PosY | Direction::NegY => [x, depth, y],
            Direction::PosZ | Direction::NegZ => [x, y, depth],
        });

        let normal = dir.get_normal();

        let mask = OcclusionMask::for_corners(corners);

        Self {
            positions,
            uvs,
            normal,
            mask,
            texture,
            render_mode,
            tintable,
        }
    }

    #[inline]
    pub fn rotate(mut self, origin: Vec3, x: f32, y: f32, z: f32) -> Self {
        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            x.to_radians(),
            y.to_radians(),
            z.to_radians(),
        );

        for position in &mut self.positions {
            let pos = Vec3::from(*position);
            let rotated = rotation * (pos - origin) + origin;

            *position = rotated.into();
        }

        let n = Vec3::from(self.normal);

        self.normal = (rotation * n).into();

        self
    }

    #[inline]
    pub fn rotate_and_recompute_mask(
        mut self,
        dir: Direction,
        origin: Vec3,
        x: f32,
        y: f32,
        z: f32,
    ) -> Self {
        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            x.to_radians(),
            y.to_radians(),
            z.to_radians(),
        );

        for position in &mut self.positions {
            let pos = Vec3::from(*position);
            let rotated = rotation * (pos - origin) + origin;

            *position = rotated.into();
        }

        let n = Vec3::from(self.normal);

        self.normal = (rotation * n).into();

        let corners = match dir {
            Direction::PosX | Direction::NegX => self.positions.map(|[_, y, z]| [z, y]),
            Direction::PosY | Direction::NegY => self.positions.map(|[x, _, z]| [z, x]),
            Direction::PosZ | Direction::NegZ => self.positions.map(|[x, y, _]| [x, y]),
        };

        self.mask = OcclusionMask::for_corners(corners);

        self
    }

    #[inline(always)]
    pub fn can_occlude(&self) -> bool {
        !self.mask.is_empty()
    }

    #[inline(always)]
    pub const fn positions(&self) -> &[[f32; 3]] {
        &self.positions
    }

    #[inline(always)]
    pub const fn normal(&self) -> &[f32; 3] {
        &self.normal
    }

    #[inline(always)]
    pub const fn uvs(&self) -> &[[f32; 2]] {
        &self.uvs
    }

    #[inline(always)]
    pub const fn mask(&self) -> OcclusionMask {
        self.mask
    }

    #[inline(always)]
    pub const fn texture(&self) -> TextureId {
        self.texture
    }

    #[inline(always)]
    pub const fn texture_raw(&self) -> u32 {
        self.texture.0
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

#[derive(Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    #[serde(rename = "east")]
    PosX = 0,
    #[serde(rename = "west")]
    NegX = 1,

    #[serde(rename = "up")]
    PosY = 2,
    #[serde(rename = "down")]
    NegY = 3,

    #[serde(rename = "south")]
    PosZ = 4,
    #[serde(rename = "north")]
    NegZ = 5,
}

impl Direction {
    #[inline(always)]
    pub const fn get_normal(self) -> [f32; 3] {
        match self {
            Direction::PosX => [1.0, 0.0, 0.0],
            Direction::NegX => [-1.0, 0.0, 0.0],
            Direction::PosY => [0.0, 1.0, 0.0],
            Direction::NegY => [0.0, -1.0, 0.0],
            Direction::PosZ => [0.0, 0.0, 1.0],
            Direction::NegZ => [0.0, 0.0, -1.0],
        }
    }
}

impl Display for Direction {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::PosX => f.write_str("east"),
            Direction::NegX => f.write_str("west"),
            Direction::PosY => f.write_str("up"),
            Direction::NegY => f.write_str("down"),
            Direction::PosZ => f.write_str("south"),
            Direction::NegZ => f.write_str("north"),
        }
    }
}

impl Not for Direction {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        match self {
            Direction::PosX => Direction::NegX,
            Direction::NegX => Direction::PosX,
            Direction::PosY => Direction::NegY,
            Direction::NegY => Direction::PosY,
            Direction::PosZ => Direction::NegZ,
            Direction::NegZ => Direction::PosZ,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RenderMode {
    Opaque = 0,
    Cutout = 1,
    Translucent = 2,
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
