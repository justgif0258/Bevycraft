use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use crate::prelude::{BlockBehaviour, BlockFlags};
use bevy::math::bounding::Aabb3d;
use builder_pattern::Builder;
use bevycraft_core::prelude::Recordable;

const DEFAULT_MODEL_PATH: &'static str = "models/block";

const DEFAULT_TEXTURES_PATH: &'static str = "textures/block";

#[derive(Builder, Debug, PartialEq)]
pub struct Block {
    #[into]
    #[public]
    #[default(BlockBehaviour::default())]
    behaviour: BlockBehaviour,

    #[into]
    #[public]
    #[default(Box::new([]))]
    shapes: Box<[Aabb3d]>,

    #[into]
    #[public]
    #[default(Attachments::new())]
    attachments: Attachments,
    
    #[into]
    #[public]
    #[default(DEFAULT_MODEL_PATH.into())]
    model: PathBuf,
    
    #[into]
    #[public]
    #[default(DEFAULT_TEXTURES_PATH.into())]
    textures: PathBuf,
}

impl Block {
    #[must_use]
    #[inline(always)]
    pub const fn air(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::AIR)
    }

    #[inline(always)]
    pub const fn hardness(&self) -> f32 {
        self.behaviour.hardness
    }

    #[inline(always)]
    pub const fn toughness(&self) -> f32 {
        self.behaviour.toughness
    }

    #[inline(always)]
    pub const fn friction(&self) -> f32 {
        self.behaviour.friction
    }

    #[inline(always)]
    pub const fn viscosity(&self) -> f32 {
        self.behaviour.viscosity
    }

    #[inline(always)]
    pub const fn collidable(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::COLLIDABLE)
    }

    #[inline(always)]
    pub const fn occludable(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::OCCLUDABLE)
    }

    #[inline(always)]
    pub const fn greedy_meshable(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::GREEDY_MESHABLE)
    }

    #[inline(always)]
    pub const fn opaque(&self) -> bool {
        !self.behaviour.flags.contains(BlockFlags::CUTOUT)
            && !self.behaviour.flags.contains(BlockFlags::TRANSLUCENT)
    }

    #[inline(always)]
    pub const fn cutout(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::CUTOUT)
    }

    #[inline(always)]
    pub const fn translucent(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::TRANSLUCENT)
    }

    #[inline(always)]
    pub const fn replaceable(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::REPLACEABLE)
    }

    #[inline(always)]
    pub const fn can_support(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::CAN_SUPPORT)
    }

    #[inline(always)]
    pub const fn does_connect(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::DOES_CONNECT)
    }

    #[inline(always)]
    pub const fn does_spawn(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::DOES_SPAWN)
    }

    #[inline(always)]
    pub const fn climbable(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::CLIMBABLE)
    }

    #[inline(always)]
    pub const fn passable(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::PASSABLE)
    }

    #[inline(always)]
    pub const fn shapes(&self) -> &[Aabb3d] {
        &self.shapes
    }

    #[inline(always)]
    pub fn get_attachment<T: Recordable>(&self, attachment: AttachmentAttribute<T>) -> Option<&T> {
        self.attachments.get(attachment)
    }
    
    #[inline(always)]
    pub fn model_path(&self) -> &str {
        self.model.to_str().unwrap_or(DEFAULT_MODEL_PATH)
    }
    
    #[inline(always)]
    pub fn textures_path(&self) -> impl AsRef<Path> {
        self.textures.to_str().unwrap_or(DEFAULT_TEXTURES_PATH)
    }
}

#[derive(Debug, PartialEq)]
pub struct Attachments(BTreeMap<&'static str, Box<dyn Recordable>>);

impl Attachments {
    #[inline]
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    #[inline]
    pub fn attach<T: Recordable>(mut self, attachment: AttachmentAttribute<T>, value: T) {
        assert!(!self.0.contains_key(&attachment.name), "Duplicate attachment name");

        self.0.insert(attachment.name, Box::new(value));
    }

    #[inline(always)]
    pub fn get<T: Recordable>(&self, attachment: AttachmentAttribute<T>) -> Option<&T> {
        self.0.get(&attachment.name)
            .map(|b|
                b.as_ref()
                    .downcast_ref::<T>()
                    .unwrap()
            )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttachmentAttribute<T: Recordable> {
    name    : &'static str,
    _marker : PhantomData<T>,
}

impl<T: Recordable> AttachmentAttribute<T> {
    #[inline(always)]
    pub const fn new(name: &'static str) -> Self {
        Self { name, _marker : PhantomData }
    }
}