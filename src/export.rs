pub use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub use stable_deref_trait::StableDeref;

pub type NotSendOrSync = PhantomData<*const ()>;
