use crate::prelude::{Direction, OcclusionMask, RenderMode, TextureId, Vertex};

#[derive(Debug, Clone, PartialEq)]
pub struct OccludableQuad {
    vertices: [Vertex; 4],
    mask: OcclusionMask,
    render_mode: RenderMode,
    tintable: bool,
}

impl OccludableQuad {
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

        let mut corners = [
            ([x0, y0], [u0, v1]),
            ([x1, y0], [u1, v1]),
            ([x1, y1], [u1, v0]),
            ([x0, y1], [u0, v0]),
        ];

        if matches!(dir, Direction::NegX | Direction::NegY | Direction::NegZ) {
            corners.swap(1, 3);
        }

        let normal = dir.get_normal();

        let vertices = corners.map(|([x, y], uv)| match dir {
            Direction::PosX | Direction::NegX => Vertex {
                position: [depth, y, x],
                uv,
                normal,
                texture,
            },
            Direction::PosY | Direction::NegY => Vertex {
                position: [x, depth, y],
                uv,
                normal,
                texture,
            },
            Direction::PosZ | Direction::NegZ => Vertex {
                position: [x, y, depth],
                uv,
                normal,
                texture,
            },
        });

        let mask = OcclusionMask::for_points([[x0, y0], [x1, y0], [x1, y1], [x0, y1]]);

        Self {
            vertices,
            mask,
            render_mode,
            tintable,
        }
    }
}
