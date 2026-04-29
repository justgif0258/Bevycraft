use bevy::{
    asset::{Assets, Handle, RenderAssetUsages},
    ecs::resource::Resource,
    image::{Image, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    platform::collections::HashMap,
    render::render_resource::{
        Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
    },
    utils::default,
};
use bevycraft_core::prelude::AssetLocation;
use image::ImageReader;
use rapidhash::fast::RandomState;

use crate::prelude::{RenderMode, VertexMaterial};

#[derive(Resource)]
pub struct ArrayTexture {
    texture_lut: HashMap<AssetLocation, TextureId, RandomState>,
    materials: Option<[Handle<VertexMaterial>; 3]>,

    storage: Option<Vec<u8>>,

    width: u32,
    height: u32,

    init: bool,
}

impl ArrayTexture {
    #[inline]
    pub fn new_uninit(width: u32, height: u32) -> Self {
        assert!(width.is_power_of_two());

        Self {
            texture_lut: HashMap::with_hasher(RandomState::new()),
            materials: None,
            storage: Some(Vec::new()),
            width,
            height,
            init: false,
        }
    }

    #[inline]
    pub fn load_from_asset_location(&mut self, location: &AssetLocation) {
        assert!(!self.init, "Cannot modify an initialized array texture");

        if self.texture_lut.contains_key(location) {
            return;
        }

        let path = format!(
            "bevycraft_app/assets/{}/textures/{}.png",
            location.namespace(),
            location.path()
        );

        let reader = ImageReader::open(path).unwrap();

        let img = reader.decode().unwrap().into_rgba8();

        assert_eq!(
            img.width(),
            self.width,
            "Texture width does not match array texture width"
        );

        assert_eq!(
            img.height(),
            self.height,
            "Texture height does not match array texture height"
        );

        self.load_bytes_with_location(location.clone(), img.into_iter());
    }

    #[inline]
    pub fn load_bytes_with_location<'a>(
        &mut self,
        location: AssetLocation,
        bytes: impl Iterator<Item = &'a u8>,
    ) {
        assert!(!self.init, "Cannot modify an initialized array texture");
        assert!(
            !self.texture_lut.contains_key(&location),
            "Tried pushing duplicate texture '{}'",
            location
        );

        let storage = self.storage.as_mut().unwrap();

        storage.extend(bytes);

        self.texture_lut
            .insert(location, TextureId(self.texture_lut.len() as u32));
    }

    #[inline]
    pub fn init_array(&mut self, images: &mut Assets<Image>, mats: &mut Assets<VertexMaterial>) {
        assert!(!self.init, "Attempted double initializing array texture");

        let mut array_image = Image::new(
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: self.texture_lut.len() as u32,
            },
            TextureDimension::D2,
            self.storage.take().unwrap(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
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

        self.materials = Some([
            mats.add(VertexMaterial {
                array_texture: image.clone(),
                render_mode: RenderMode::Opaque,
            }),
            mats.add(VertexMaterial {
                array_texture: image.clone(),
                render_mode: RenderMode::Cutout,
            }),
            mats.add(VertexMaterial {
                array_texture: image,
                render_mode: RenderMode::Translucent,
            }),
        ]);

        self.init = true;
    }

    #[inline(always)]
    #[must_use]
    pub fn get_vertex_material(&self, mode: RenderMode) -> Handle<VertexMaterial> {
        assert!(self.init, "Tried getting an uninitialized vertex material");

        let materials = self
            .materials
            .as_ref()
            .expect("Tried accessing material on uninitialized array texture");

        materials[mode as usize].clone()
    }

    #[inline(always)]
    #[must_use]
    pub fn get_texture_id(&self, location: &AssetLocation) -> Option<TextureId> {
        self.texture_lut.get(location).copied()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (&AssetLocation, &TextureId)> {
        assert!(
            self.init,
            "Tried iterating over an uninitialized array texture"
        );

        self.texture_lut.iter()
    }

    #[inline(always)]
    pub const fn is_init(&self) -> bool {
        self.init
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureId(pub(crate) u32);
