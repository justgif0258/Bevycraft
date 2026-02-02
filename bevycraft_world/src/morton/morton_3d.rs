use bevy::math::{I64Vec3, U64Vec3};

pub trait MortonEncodable {
    fn encode_u64(&self) -> u64;
}

pub trait MortonDecodable {
    fn decode_u64(morton: u64) -> Self;
}

const MORTON_INDEX_MASK: u64 = 0x7;

#[derive(Debug)]
pub struct Morton3D(u64);

impl Morton3D {
    pub fn encode(value: impl MortonEncodable) -> Self {
        Self(value.encode_u64())
    }

    #[inline]
    pub fn decode<T: MortonDecodable>(&self) -> T {
        T::decode_u64(self.0)
    }

    #[inline]
    pub fn get_morton_index(&self, index: usize) -> usize {
        ((self.0 >> index * 3) & MORTON_INDEX_MASK) as usize
    }

    #[inline]
    fn split_bits(n: impl Into<u64>) -> u64 {
        let mut x = n.into() & 0x1fffff;
        x = (x | x << 32) & 0x1f00000000ffff;
        x = (x | x << 16) & 0x1f0000ff0000ff;
        x = (x | x << 8) & 0x100f00f00f00f00f;
        x = (x | x << 4) & 0x10c30c30c30c30c3;
        (x | x << 2) & 0x1249249249249249
    }

    #[inline]
    fn join_bits(n: impl Into<u64>) -> u64 {
        let mut x = n.into() & 0x1249249249249249;
        x = (x ^ x >> 2) & 0x10c30c30c30c30c3;
        x = (x ^ x >> 4) & 0x100f00f00f00f00f;
        x = (x ^ x >> 8) & 0x1f0000ff0000ff;
        x = (x ^ x >> 16) & 0x1f00000000ffff;
        (x ^ x >> 32) & 0x1fffff
    }
}

impl MortonEncodable for I64Vec3 {
    fn encode_u64(&self) -> u64 {
        Morton3D::split_bits(self.x.unsigned_abs())
            | (Morton3D::split_bits(self.y.unsigned_abs()) << 1)
            | (Morton3D::split_bits(self.z.unsigned_abs()) << 2)
    }
}

impl MortonDecodable for I64Vec3 {
    fn decode_u64(morton: u64) -> Self {
        I64Vec3::new(
            Morton3D::join_bits(morton) as i64,
            Morton3D::join_bits(morton >> 1) as i64,
            Morton3D::join_bits(morton >> 2) as i64
        )
    }
}

impl MortonEncodable for U64Vec3 {
    fn encode_u64(&self) -> u64 {
        Morton3D::split_bits(self.x)
            | (Morton3D::split_bits(self.y) << 1)
            | (Morton3D::split_bits(self.z) << 2)
    }
}

impl MortonDecodable for U64Vec3 {
    fn decode_u64(morton: u64) -> Self {
        U64Vec3::new(
            Morton3D::join_bits(morton),
            Morton3D::join_bits(morton >> 1),
            Morton3D::join_bits(morton >> 2)
        )
    }
}

impl MortonEncodable for [i64; 3] {
    fn encode_u64(&self) -> u64 {
        Morton3D::split_bits(self[0].unsigned_abs())
            | (Morton3D::split_bits(self[1].unsigned_abs()) << 1)
            | (Morton3D::split_bits(self[2].unsigned_abs()) << 2)
    }
}

impl MortonDecodable for [i64; 3] {
    fn decode_u64(morton: u64) -> Self {
        [
            Morton3D::join_bits(morton) as i64,
            Morton3D::join_bits(morton >> 1) as i64,
            Morton3D::join_bits(morton >> 2) as i64,
        ]
    }
}

impl MortonEncodable for (i64, i64, i64) {
    fn encode_u64(&self) -> u64 {
        Morton3D::split_bits(self.0.unsigned_abs())
            | (Morton3D::split_bits(self.1.unsigned_abs()) << 1)
            | (Morton3D::split_bits(self.2.unsigned_abs()) << 2)
    }
}

impl MortonDecodable for (i64, i64, i64) {
    fn decode_u64(morton: u64) -> Self {
        (
            Morton3D::join_bits(morton) as i64,
            Morton3D::join_bits(morton >> 1) as i64,
            Morton3D::join_bits(morton >> 2) as i64,
        )
    }
}

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