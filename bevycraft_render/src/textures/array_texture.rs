use bevy::asset::RenderAssetUsages;
use bevy::image::*;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use frozen_collections::FzHashMap;
use frozen_collections::maps::Iter;
use bevycraft_core::prelude::AssetLocation;
use crate::prelude::{RenderMode, TextureId, VertexMaterial};

#[derive(Resource)]
pub struct ArrayTexture {
    texture_lut : FzHashMap<AssetLocation, TextureId>,
    materials   : [Handle<VertexMaterial>; 3],
    image       : Handle<Image>,
}

impl ArrayTexture {
    #[inline]
    #[must_use]
    pub fn get_vertex_material(&self, mode: RenderMode) -> Handle<VertexMaterial> {
        self.materials[mode as usize].clone()
    }

    #[inline]
    #[must_use]
    pub const fn builder(resolution: u32) -> ArrayTextureBuilder {
        assert!(resolution.is_power_of_two(), "The resolution of an array texture must be a power of 2");

        ArrayTextureBuilder {
            textures: Vec::new(),
            names: Vec::new(),
            resolution,
        }
    }

    #[inline(always)]
    pub fn get_texture_id(&self, location: &AssetLocation) -> Option<TextureId> {
        self.texture_lut.get(location).copied()
    }

    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, AssetLocation, TextureId> {
        self.texture_lut.iter()
    }
}

pub struct ArrayTextureBuilder {
    textures    : Vec<Vec<u8>>,
    names       : Vec<AssetLocation>,
    resolution  : u32,
}

impl ArrayTextureBuilder {
    #[inline]
    pub fn register(&mut self, name: AssetLocation, pixels: Vec<u8>) -> TextureId {
        let id = TextureId(self.textures.len() as u32);

        self.textures.push(pixels);
        self.names.push(name);

        id
    }

    pub fn build_and_send(
        self,
        mats:   &mut Assets<VertexMaterial>,
        images: &mut Assets<Image>,
    ) -> ArrayTexture {
        let layer_count = self.textures.len() as u32;

        let mut all_pixels = Vec::with_capacity((8 * 8 * 4 * layer_count) as usize);

        for texture in &self.textures {
            all_pixels.extend_from_slice(texture);
        }

        let mut array_image = Image::new(
            Extent3d {
                width: self.resolution,
                height: self.resolution,
                depth_or_array_layers: layer_count,
            },
            TextureDimension::D2,
            all_pixels,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
        );

        array_image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2Array),
            ..default()
        });

        array_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            mag_filter: ImageFilterMode::Nearest,
            min_filter: ImageFilterMode::Nearest,
            mipmap_filter: ImageFilterMode::Nearest,
            ..default()
        });

        let image = images.add(array_image);

        let texture_lut = FzHashMap::from_iter(
            self.names.into_iter()
                .enumerate()
                .map(|(i, k)| (k, TextureId(i as u32)))
        );

        ArrayTexture {
            texture_lut,
            materials: [
                mats.add(VertexMaterial { array_texture: image.clone(), render_mode: RenderMode::Opaque }),
                mats.add(VertexMaterial { array_texture: image.clone(), render_mode: RenderMode::Cutout }),
                mats.add(VertexMaterial { array_texture: image.clone(), render_mode: RenderMode::Translucent }),
            ],
            image,
        }
    }
}