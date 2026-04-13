use bevy::render::render_resource::{Extent3d, Origin3d, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use frozen_collections::FzStringMap;
use crate::prelude::TextureId;

pub struct ArrayTexture {
    texture_lut : FzStringMap<Box<str>, TextureId>,
    texture     : Texture,
    view        : TextureView,
}

impl ArrayTexture {
    #[inline]
    #[must_use]
    pub const fn builder(resolution: usize) -> ArrayTextureBuilder {
        assert!(resolution.is_power_of_two(), "The resolution of an array texture must be a power of 2");

        ArrayTextureBuilder {
            textures: Vec::new(),
            names: Vec::new(),
            resolution,
        }
    }

    #[inline(always)]
    pub fn get_texture_id(&self, name: &str) -> Option<TextureId> {
        self.texture_lut.get(name).copied()
    }
}

pub struct ArrayTextureBuilder {
    textures    : Vec<Vec<u8>>,
    names       : Vec<String>,
    resolution  : usize,
}

impl ArrayTextureBuilder {
    #[inline]
    pub fn register(&mut self, name: impl Into<String>, pixels: Vec<u8>) -> TextureId {
        let resolution = (pixels.len() / 4).ilog(2) as usize;
        let name = name.into();

        assert_eq!(self.resolution, resolution, "Resolution mismatch while trying to register '{}' texture", &name);

        let id = TextureId(self.textures.len() as u32);

        self.textures.push(pixels);
        self.names.push(name);

        id
    }

    pub fn build_and_send(
        self,
        device: &mut RenderDevice,
        queue: &mut RenderQueue,
    ) -> ArrayTexture {
        let layer_count = self.textures.len() as u32;
        let texture_size = Extent3d {
            width: 8,
            height: 8,
            depth_or_array_layers: layer_count,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("blocks_array"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for (layer, pixels) in self.textures.iter().enumerate() {
            queue.write_texture(
                TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: Origin3d { x: 0, y: 0, z: layer as u32 },
                    aspect: TextureAspect::All,
                },
                pixels,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(8 * 4),
                    rows_per_image: Some(8),
                },
                Extent3d {
                    width: 8,
                    height: 8,
                    depth_or_array_layers: 1,
                },
            );
        }

        let view = texture.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });

        let name_map = FzStringMap::from_iter(
            self.names
                .into_iter()
                .enumerate()
                .map(|(i, name)| (name.into_boxed_str(), TextureId(i as u32))),
        );

        ArrayTexture { texture_lut: name_map, texture, view }
    }
}