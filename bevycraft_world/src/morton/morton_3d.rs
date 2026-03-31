use bevy::math::*;
use std::arch::x86_64::{_pdep_u64, _pext_u64};
use std::hash::{Hash, Hasher};
use std::ops::{BitAnd, Shl, Shr};

const MASK_X: u64 = 0x1249249249249249;
const MASK_Y: u64 = MASK_X << 1;
const MASK_Z: u64 = MASK_X << 2;

const MASK_YZ: u64 = !MASK_X;
const MASK_XZ: u64 = !MASK_Y;
const MASK_XY: u64 = !MASK_Z;

pub trait MortonEncodable {
    fn encode_x(&self) -> u32;

    fn encode_y(&self) -> u32;

    fn encode_z(&self) -> u32;
}

pub trait MortonDecodable {
    fn decode(x: u64, y: u64, z: u64) -> Self;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Morton3D(u64);

impl From<u64> for Morton3D {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Morton3D> for u64 {
    fn from(value: Morton3D) -> Self {
        value.0
    }
}

impl Hash for Morton3D {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0)
    }
}

impl Morton3D {
    #[inline]
    pub fn encode<T: MortonEncodable>(value: T) -> Self {
        unsafe { Self::interleave(value) }
    }

    #[inline]
    pub fn decode<T: MortonDecodable>(&self) -> T {
        unsafe { self.deinterleave() }
    }

    #[inline]
    pub const fn inc_x(&mut self) {
        self.0 = ((self.0 | MASK_YZ).wrapping_add(1) & MASK_X) | (self.0 & MASK_YZ);
    }

    #[inline]
    pub const fn dec_x(&mut self) {
        self.0 = ((self.0 & MASK_X).wrapping_sub(1) & MASK_X) | (self.0 & MASK_YZ);
    }

    #[inline]
    pub const fn inc_y(&mut self) {
        self.0 = ((self.0 | MASK_XZ).wrapping_add(1) & MASK_Y) | (self.0 & MASK_XZ);
    }

    #[inline]
    pub const fn dec_y(&mut self) {
        self.0 = ((self.0) & MASK_Y).wrapping_sub(2) & MASK_Y | (self.0 & MASK_XZ);
    }

    #[inline]
    pub const fn inc_z(&mut self) {
        self.0 = ((self.0 | MASK_XY).wrapping_add(1) & MASK_Z) | (self.0 & MASK_XY);
    }

    #[inline]
    pub const fn dec_z(&mut self) {
        self.0 = ((self.0 & MASK_Z).wrapping_sub(4) & MASK_Z) | (self.0 & MASK_XY);
    }

    #[inline]
    pub const fn raw(&self) -> u64 {
        self.0
    }

    #[inline]
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "bmi2")]
    fn interleave<T: MortonEncodable>(value: T) -> Self {
        Self(
            _pdep_u64(value.encode_x() as u64, MASK_X)
                | _pdep_u64(value.encode_y() as u64, MASK_Y)
                | _pdep_u64(value.encode_z() as u64, MASK_Z),
        )
    }

    #[inline]
    #[cfg(target_arch = "x86")]
    fn interleave<T: MortonEncodable>(value: T) -> Self {
        Self(
            Morton3D::split_bits(value.encode_x())
                | (Morton3D::split_bits(value.encode_y()) << 1)
                | (Morton3D::split_bits(value.encode_z()) << 2),
        )
    }

    #[inline]
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "bmi2")]
    fn deinterleave<T: MortonDecodable>(&self) -> T {
        T::decode(
            _pext_u64(self.0, MASK_X),
            _pext_u64(self.0, MASK_Y),
            _pext_u64(self.0, MASK_Z),
        )
    }

    #[inline]
    #[cfg(target_arch = "x86")]
    fn deinterleave<T: MortonDecodable>(&self) -> T {
        T::decode(
            Self::join_bits(self.0) as u32,
            Self::join_bits(self.0 >> 1) as u32,
            Self::join_bits(self.0 >> 2) as u32,
        )
    }

    #[inline]
    #[cfg(target_arch = "x86")]
    fn split_bits(n: impl Into<u64>) -> u64 {
        let mut x = n.into() & 0x1fffff;
        x = (x | x << 32) & 0x1f00000000ffff;
        x = (x | x << 16) & 0x1f0000ff0000ff;
        x = (x | x << 8) & 0x100f00f00f00f00f;
        x = (x | x << 4) & 0x10c30c30c30c30c3;
        (x | x << 2) & 0x1249249249249249
    }

    #[inline]
    #[cfg(target_arch = "x86")]
    fn join_bits(n: impl Into<u64>) -> u64 {
        let mut x = n.into() & 0x1249249249249249;
        x = (x ^ x >> 2) & 0x10c30c30c30c30c3;
        x = (x ^ x >> 4) & 0x100f00f00f00f00f;
        x = (x ^ x >> 8) & 0x1f0000ff0000ff;
        x = (x ^ x >> 16) & 0x1f00000000ffff;
        (x ^ x >> 32) & 0x1fffff
    }
}

impl Shl<usize> for Morton3D {
    type Output = Self;

    #[inline]
    fn shl(self, rhs: usize) -> Self::Output {
        Self(self.0.shl(rhs))
    }
}

impl Shl<&usize> for Morton3D {
    type Output = Self;

    #[inline]
    fn shl(self, rhs: &usize) -> Self::Output {
        Self(self.0.shl(*rhs))
    }
}

impl Shl<usize> for &Morton3D {
    type Output = Morton3D;

    #[inline]
    fn shl(self, rhs: usize) -> Self::Output {
        (*self).shl(rhs)
    }
}

impl Shl<&usize> for &Morton3D {
    type Output = Morton3D;

    #[inline]
    fn shl(self, rhs: &usize) -> Self::Output {
        (*self).shl(*rhs)
    }
}

impl Shr<usize> for Morton3D {
    type Output = Self;

    #[inline]
    fn shr(self, rhs: usize) -> Self::Output {
        Self(self.0.shr(rhs))
    }
}

impl Shr<&usize> for Morton3D {
    type Output = Self;

    #[inline]
    fn shr(self, rhs: &usize) -> Self::Output {
        Self(self.0.shr(*rhs))
    }
}

impl Shr<usize> for &Morton3D {
    type Output = Morton3D;

    #[inline]
    fn shr(self, rhs: usize) -> Self::Output {
        (*self).shr(rhs)
    }
}

impl Shr<&usize> for &Morton3D {
    type Output = Morton3D;

    #[inline]
    fn shr(self, rhs: &usize) -> Self::Output {
        (*self).shr(*rhs)
    }
}

impl BitAnd<u64> for Morton3D {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: u64) -> Self::Output {
        Self(self.0.bitand(rhs))
    }
}

impl BitAnd<&u64> for Morton3D {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: &u64) -> Self::Output {
        Self(self.0.bitand(*rhs))
    }
}

impl BitAnd<u64> for &Morton3D {
    type Output = Morton3D;

    #[inline]
    fn bitand(self, rhs: u64) -> Self::Output {
        (*self).bitand(rhs)
    }
}

impl BitAnd<&u64> for &Morton3D {
    type Output = Morton3D;

    #[inline]
    fn bitand(self, rhs: &u64) -> Self::Output {
        (*self).bitand(*rhs)
    }
}

impl MortonEncodable for IVec3 {
    #[inline]
    fn encode_x(&self) -> u32 {
        self.x.unsigned_abs()
    }

    #[inline]
    fn encode_y(&self) -> u32 {
        self.y.unsigned_abs()
    }

    #[inline]
    fn encode_z(&self) -> u32 {
        self.z.unsigned_abs()
    }
}

impl MortonEncodable for UVec3 {
    fn encode_x(&self) -> u32 {
        self.x
    }

    fn encode_y(&self) -> u32 {
        self.y
    }

    fn encode_z(&self) -> u32 {
        self.z
    }
}

impl MortonDecodable for UVec3 {
    #[inline]
    fn decode(x: u64, y: u64, z: u64) -> Self {
        Self::new(x as _, y as _, z as _)
    }
}

impl MortonEncodable for [u32; 3] {
    #[inline]
    fn encode_x(&self) -> u32 {
        self[0]
    }

    #[inline]
    fn encode_y(&self) -> u32 {
        self[1]
    }

    #[inline]
    fn encode_z(&self) -> u32 {
        self[2]
    }
}

impl MortonDecodable for [u32; 3] {
    #[inline]
    fn decode(x: u64, y: u64, z: u64) -> Self {
        [x as _, y as _, z as _]
    }
}

impl MortonEncodable for (u32, u32, u32) {
    fn encode_x(&self) -> u32 {
        self.0
    }

    fn encode_y(&self) -> u32 {
        self.1
    }

    fn encode_z(&self) -> u32 {
        self.2
    }
}

impl MortonDecodable for (u32, u32, u32) {
    #[inline]
    fn decode(x: u64, y: u64, z: u64) -> Self {
        (x as _, y as _, z as _)
    }
}
