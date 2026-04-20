use std::collections::BTreeMap;
use std::marker::PhantomData;
use crate::prelude::BlockBehaviour;
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
    attachments: Attachments
}

impl Block {
    #[must_use]
    #[inline(always)]
    pub const fn behaviour(&self) -> &BlockBehaviour {
        &self.behaviour
    }

    #[must_use]
    #[inline(always)]
    pub const fn shapes(&self) -> &[Aabb3d] {
        &self.shapes
    }

    #[must_use]
    #[inline(always)]
    pub fn get_attachment<T: Recordable>(&self, attachment: AttachmentAttribute<T>) -> Option<&T> {
        self.attachments.get(attachment)
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

#[derive(Debug, Clone, Eq, PartialEq)]
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