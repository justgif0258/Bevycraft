use bevycraft_core::prelude::Recordable;
use crate::prelude::{BlockDefinition, BlockPhysics};

pub trait Block: Recordable {
    fn definitions(&self) -> BlockDefinition;
    
    fn physics(&self) -> BlockPhysics;
}