use const_builder::ConstBuilder;

#[derive(Debug)]
pub struct Block {
    properties: BlockBehaviour,
}

impl Block {
    pub const fn new(behaviour: BlockBehaviour) -> Self {
        Self { properties: behaviour }
    }
}

impl BehaviourTrait for Block {
    #[inline]
    fn hardness(&self) -> f32 {
        self.properties.hardness
    }

    #[inline]
    fn toughness(&self) -> f32 {
        self.properties.toughness
    }

    #[inline]
    fn friction(&self) -> f32 {
        self.properties.friction
    }

    #[inline]
    fn emission(&self) -> f32 {
        self.properties.emission
    }

    #[inline]
    fn light_influence(&self) -> i32 {
        self.properties.light_influence.cast_signed()
    }

    #[inline]
    fn translucent(&self) -> bool {
        self.properties.translucent
    }

    #[inline]
    fn replaceable(&self) -> bool {
        self.properties.replaceable
    }

    #[inline]
    fn occludable(&self) -> bool {
        self.properties.occludable
    }

    #[inline]
    fn air(&self) -> bool {
        self.properties.air
    }
}

#[derive(ConstBuilder, Debug)]
#[builder(rename = "BehaviourBuilder")]
pub struct BlockBehaviour {
    #[builder(default = 1.0)]
    pub(super) hardness: f32,

    #[builder(default = 1.0)]
    pub(super) toughness: f32,

    #[builder(default = 0.6)]
    pub(super) friction: f32,

    #[builder(default = 0.0)]
    pub(super) emission: f32,

    #[builder(default = 0)]
    pub(super) light_influence: u32,

    #[builder(default = false)]
    pub(super) translucent: bool,

    #[builder(default = false)]
    pub(super) replaceable: bool,

    #[builder(default = true)]
    pub(super) occludable: bool,

    #[builder(default = false)]
    pub(super) air: bool,
}

pub trait BehaviourTrait {
    fn hardness(&self) -> f32;

    fn toughness(&self) -> f32;

    fn friction(&self) -> f32;

    fn emission(&self) -> f32;

    fn light_influence(&self) -> i32;

    fn translucent(&self) -> bool;

    fn replaceable(&self) -> bool;

    fn occludable(&self) -> bool;

    fn air(&self) -> bool;
}