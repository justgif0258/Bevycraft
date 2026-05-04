use geo::{Intersects, LineString, Polygon, Rect, coord};

const GRID_RESOLUTION: usize = 8;
const CELL_SIZE: f32 = 0.125;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OcclusionMask(u64);

impl OcclusionMask {
    pub fn new(points: [[f32; 2]; 4]) -> Self {
        let poly: Polygon<f32> = Polygon::new(
            LineString::new(vec![
                points[0].into(),
                points[1].into(),
                points[2].into(),
                points[3].into(),
                points[0].into(),
            ]),
            Vec::new(),
        );

        let mut mask = 0u64;

        for x in 0..GRID_RESOLUTION {
            for y in 0..GRID_RESOLUTION {
                let min_x = x as f32 * CELL_SIZE;
                let min_y = y as f32 * CELL_SIZE;
                let max_x = min_x + CELL_SIZE;
                let max_y = min_y + CELL_SIZE;

                let cell = Rect::new(coord! {x: min_x, y: min_y}, coord! {x: max_x, y: max_y});

                if poly.intersects(&cell) {
                    let bit_index = x + (y * GRID_RESOLUTION);
                    mask |= 1 << bit_index;
                }
            }
        }

        Self(mask)
    }

    #[inline(always)]
    pub const fn occludes(&self, other: &Self) -> bool {
        self.0 & other.0 != 0
    }

    #[inline(always)]
    pub const fn is_occluded_by(&self, other: &Self) -> bool {
        other.occludes(self)
    }
}
