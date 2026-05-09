extern crate core;

mod block;
pub mod consts;
mod memory;
mod registries;
mod util;

pub mod prelude {
    pub use crate::block::{behaviour::*, block::*, flags::*, shape::BlockShape};
    pub use crate::memory::pattern_container::{PatternContainer, PatternIter};
    pub use crate::registries::{
        asset_location::*, defaulted_registry::*, erased_registry::*, game_registries::*,
        holder::Holder, ordered_registry::*, registrar::*, registry::*,
    };
}

pub mod blocks {
    pub use crate::block::blocks::*;
}

/*
#[macro_export]
macro_rules! register {
    ( $( $vis:vis static $name:ident : $type:ty = register($key:expr, $def:expr); )* ) => {
        $(
            $vis static $name: Holder<$type> = Holder::new();
        )*

        const _: () = {
            use ctor::ctor;
            use $crate::prelude::{AssetLocation, Registrar};

            #[ctor(unsafe)]
            fn __register() {
                $(
                    {
                        let mut reg = <$type as Registrar>::write_to_registry();
                        let location = AssetLocation::parse($key);

                        reg.register(location.clone(), $def)
                            .expect("Registration failed");

                        let id = reg.key_to_idx(&location).unwrap();
                        $name.set(id);
                    }
                )*
            }
        };
    }
}
*/
