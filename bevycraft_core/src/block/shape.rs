use bevy::math::{
    Mat3, Quat, Vec3A,
    bounding::{Aabb3d, BoundingVolume, IntersectsVolume},
};

#[derive(Debug, Clone, PartialEq)]
pub enum BlockShape {
    None,
    Single(Aabb3d),
    Multi {
        broad_bounds: Aabb3d,
        bboxes: Box<[Aabb3d]>,
    },
}

impl From<Aabb3d> for BlockShape {
    fn from(bbox: Aabb3d) -> Self {
        BlockShape::Single(bbox)
    }
}

impl<'a> From<&'a Aabb3d> for BlockShape {
    #[inline]
    fn from(bbox: &'a Aabb3d) -> Self {
        Self::Single(*bbox)
    }
}

impl<const N: usize> From<[Aabb3d; N]> for BlockShape {
    #[inline]
    fn from(bboxes: [Aabb3d; N]) -> Self {
        if N == 0 {
            Self::Single(bboxes[0])
        } else if N == 1 {
            Self::Single(bboxes[0])
        } else {
            Self::Multi {
                broad_bounds: compute_bounds(&bboxes),
                bboxes: Box::new(bboxes),
            }
        }
    }
}

impl From<Box<[Aabb3d]>> for BlockShape {
    #[inline]
    fn from(bboxes: Box<[Aabb3d]>) -> Self {
        if bboxes.len() == 0 {
            Self::None
        } else if bboxes.len() == 1 {
            Self::Single((*bboxes)[0])
        } else {
            Self::Multi {
                broad_bounds: compute_bounds(&bboxes),
                bboxes,
            }
        }
    }
}

impl<'a> From<&'a [Aabb3d]> for BlockShape {
    #[inline]
    fn from(bboxes: &'a [Aabb3d]) -> Self {
        if bboxes.len() == 0 {
            Self::None
        } else if bboxes.len() == 1 {
            Self::Single((*bboxes)[0])
        } else {
            Self::Multi {
                broad_bounds: compute_bounds(bboxes),
                bboxes: Box::from(bboxes),
            }
        }
    }
}

impl BoundingVolume for BlockShape {
    type Translation = Vec3A;

    type Rotation = Quat;

    type HalfSize = Vec3A;

    #[inline]
    fn center(&self) -> Self::Translation {
        match self {
            Self::None => Vec3A::ZERO,
            Self::Single(bbox) => bbox.center(),
            Self::Multi { broad_bounds, .. } => broad_bounds.center(),
        }
    }

    #[inline]
    fn half_size(&self) -> Self::HalfSize {
        match self {
            Self::None => Vec3A::ZERO,
            Self::Single(bbox) => bbox.half_size(),
            Self::Multi { broad_bounds, .. } => broad_bounds.half_size(),
        }
    }

    #[inline]
    fn visible_area(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Single(bbox) => bbox.visible_area(),
            Self::Multi { broad_bounds, .. } => broad_bounds.visible_area(),
        }
    }

    #[inline]
    fn contains(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, _) | (_, Self::None) => false,
            (Self::Single(a), Self::Single(b)) => a.contains(b),
            (Self::Single(a), Self::Multi { broad_bounds, .. }) => a.contains(broad_bounds),
            (
                Self::Multi {
                    broad_bounds,
                    bboxes,
                },
                Self::Single(b),
            ) => {
                if !broad_bounds.contains(b) {
                    return false;
                }

                bboxes.iter().any(|bbox| b.contains(bbox))
            }
            (
                Self::Multi {
                    broad_bounds: a,
                    bboxes,
                },
                Self::Multi {
                    broad_bounds: b,
                    bboxes: other_bboxes,
                },
            ) => {
                if !a.contains(b) {
                    return false;
                }

                other_bboxes
                    .iter()
                    .all(|other| bboxes.iter().any(|bbox| bbox.contains(other)))
            }
        }
    }

    #[inline]
    fn merge(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::None, _) | (_, Self::None) => Self::None,
            (Self::Single(a), Self::Single(b)) => Self::Single(a.merge(b)),
            (Self::Single(a), Self::Multi { bboxes, .. })
            | (Self::Multi { bboxes, .. }, Self::Single(a)) => {
                let mut vec = bboxes.to_vec();
                vec.push(*a);

                Self::Multi {
                    broad_bounds: compute_bounds(&vec),
                    bboxes: vec.into_boxed_slice(),
                }
            }
            (
                Self::Multi { bboxes, .. },
                Self::Multi {
                    bboxes: other_bboxes,
                    ..
                },
            ) => {
                let mut vec = bboxes.to_vec();
                vec.extend(other_bboxes.iter());

                Self::Multi {
                    broad_bounds: compute_bounds(&vec),
                    bboxes: vec.into_boxed_slice(),
                }
            }
        }
    }

    #[inline]
    fn grow(&self, amount: impl Into<Self::HalfSize>) -> Self {
        match self {
            Self::None => Self::None,
            Self::Single(bbox) => Self::Single(bbox.grow(amount)),
            Self::Multi { bboxes, .. } => {
                let amount = amount.into();

                let bboxes: Box<[Aabb3d]> = bboxes.iter().map(|bbox| bbox.grow(amount)).collect();

                Self::Multi {
                    broad_bounds: compute_bounds(&bboxes),
                    bboxes,
                }
            }
        }
    }

    #[inline]
    fn shrink(&self, amount: impl Into<Self::HalfSize>) -> Self {
        match self {
            Self::None => Self::None,
            Self::Single(bbox) => Self::Single(bbox.shrink(amount)),
            Self::Multi { bboxes, .. } => {
                let amount = amount.into();

                let bboxes: Box<[Aabb3d]> = bboxes.iter().map(|bbox| bbox.shrink(amount)).collect();

                Self::Multi {
                    broad_bounds: compute_bounds(&bboxes),
                    bboxes,
                }
            }
        }
    }

    #[inline]
    fn scale_around_center(&self, scale: impl Into<Self::HalfSize>) -> Self {
        match self {
            Self::None => Self::None,
            Self::Single(bbox) => Self::Single(bbox.scale_around_center(scale)),
            Self::Multi {
                broad_bounds,
                bboxes,
            } => {
                let scale = scale.into();
                let center = broad_bounds.center();

                let bboxes: Box<[Aabb3d]> = bboxes
                    .iter()
                    .map(|bbox| {
                        let b = Aabb3d {
                            min: center - (bbox.half_size() * scale),
                            max: center + (bbox.half_size() * scale),
                        };
                        debug_assert!(b.min.cmple(b.max).all());
                        b
                    })
                    .collect();

                Self::Multi {
                    broad_bounds: compute_bounds(&bboxes),
                    bboxes,
                }
            }
        }
    }

    #[inline]
    fn transformed_by(
        mut self,
        translation: impl Into<Self::Translation>,
        rotation: impl Into<Self::Rotation>,
    ) -> Self {
        self.transform_by(translation, rotation);
        self
    }

    #[inline]
    fn transform_by(
        &mut self,
        translation: impl Into<Self::Translation>,
        rotation: impl Into<Self::Rotation>,
    ) {
        self.rotate_by(rotation);
        self.translate_by(translation);
    }

    #[inline]
    fn translate_by(&mut self, translation: impl Into<Self::Translation>) {
        match self {
            Self::None => {}
            Self::Single(bbox) => bbox.translate_by(translation),
            Self::Multi {
                broad_bounds,
                bboxes,
            } => {
                let translation = translation.into();

                broad_bounds.translate_by(translation);

                for bbox in bboxes.iter_mut() {
                    bbox.translate_by(translation);
                }
            }
        }
    }

    #[inline]
    fn rotated_by(mut self, rotation: impl Into<Self::Rotation>) -> Self {
        self.rotate_by(rotation);
        self
    }

    #[inline]
    fn rotate_by(&mut self, rotation: impl Into<Self::Rotation>) {
        match self {
            Self::None => {}
            Self::Single(bbox) => bbox.rotate_by(rotation),
            Self::Multi {
                broad_bounds,
                bboxes,
            } => {
                let rotation = rotation.into();
                let center = broad_bounds.center();

                for bbox in bboxes.iter_mut() {
                    let rot_mat = Mat3::from_quat(rotation);
                    let half_size = rot_mat.abs() * bbox.half_size();
                    *bbox = Aabb3d::new(rot_mat * center, half_size);
                }

                *broad_bounds = compute_bounds(bboxes);
            }
        };
    }
}

impl IntersectsVolume<Aabb3d> for BlockShape {
    #[inline]
    fn intersects(&self, volume: &Aabb3d) -> bool {
        match self {
            Self::None => false,
            Self::Single(bbox) => bbox.intersects(volume),
            Self::Multi {
                broad_bounds,
                bboxes,
            } => {
                if !broad_bounds.intersects(volume) {
                    return false;
                }

                bboxes.iter().any(|bbox| bbox.intersects(volume))
            }
        }
    }
}

impl IntersectsVolume<BlockShape> for Aabb3d {
    #[inline]
    fn intersects(&self, volume: &BlockShape) -> bool {
        match volume {
            BlockShape::None => false,
            BlockShape::Single(other) => self.intersects(other),
            BlockShape::Multi {
                broad_bounds,
                bboxes,
            } => {
                if !self.intersects(broad_bounds) {
                    return false;
                }

                bboxes.iter().any(|other| self.intersects(other))
            }
        }
    }
}

#[inline(always)]
fn compute_bounds(bboxes: &[Aabb3d]) -> Aabb3d {
    let mut min = bboxes[0].min;
    let mut max = bboxes[0].max;

    for bbox in bboxes.iter().skip(1) {
        min = min.min(bbox.min);
        max = max.max(bbox.max);
    }

    Aabb3d { min, max }
}
