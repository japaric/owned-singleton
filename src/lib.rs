//! Owned singletons
//!
//! An owned singleton is a proxy (struct) that grants exclusive access to a `static mut` variable.
//!
//! # Features
//!
//! Owned singletons are smaller than `&'static mut` references; they are zero sized types.
//!
//! Doesn't sound useful enough to you? The `Singleton` abstraction can be used to implement
//! [statically allocated memory pools whose handles are a single byte in size and are automatically
//! deallocated on `drop`.][alloc-singleton]
//!
//! [alloc-singleton]: https://crates.io/crates/alloc-singleton
//!
//! # Examples
//!
//! The `Singleton` attribute creates a proxy (`struct`) for the given `static mut` variable and
//! implements the `Singleton`, `Deref`, `DerefMut` and `StableDeref`s traits for it.
//!
//! ```
//! use owned_singleton::Singleton;
//!
//! #[Singleton]
//! static mut FOO: u32 = 0;
//!
//! let mut foo = unsafe { FOO::new() };
//! assert_eq!(*foo, 0);
//! *foo += 1;
//! assert_eq!(*foo, 1);
//!
//! let bar: &'static mut u32 = foo.unwrap();
//! assert_eq!(*bar, 1);
//! ```
//!
//! The `Singleton` attribute doesn't implement the `Send` or `Sync` traits by default; this results
//! in a proxy struct that does *not* implement `Send` or `Sync`. To opt into the `Send` and `Sync`
//! traits add the `Send` and `Sync` arguments to the `Singleton` attribute.
//!
//! ```
//! use owned_singleton::Singleton;
//!
//! #[Singleton(Send, Sync)]
//! static mut FOO: u32 = 0;
//!
//! fn is_send<T>() where T: Send {}
//! fn is_sync<T>() where T: Sync {}
//!
//! is_send::<FOO>();
//! is_sync::<FOO>();
//! ```
//!
//! Using `Singleton` on a `static` variable results in `DerefMut` not being implemented for the
//! proxy struct. However, the proxy struct will still be a handle to a `static mut` variable so
//! there's *no* `Sync` requirement on the type of the `static mut` variable.
//!
//! ```
//! use std::marker::PhantomData;
//!
//! use owned_singleton::Singleton;
//!
//! // `PhantomData<*const ()>` does not implement `Send` or `Sync`
//! #[Singleton]
//! static FOO: PhantomData<*const ()> = PhantomData;
//! ```

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

extern crate owned_singleton_macros;
extern crate stable_deref_trait;

pub use owned_singleton_macros::Singleton;
use stable_deref_trait::StableDeref;

#[doc(hidden)]
pub mod export;

/// An owned singleton: a proxy (handle) for a `static mut` variable
pub unsafe trait Singleton: StableDeref {
    /// The type of the `static mut` variable behind this proxy
    type Type;

    /// Creates a new singleton
    ///
    /// # Unsafety
    ///
    /// It's UB to create more than one instance of this singleton
    unsafe fn new() -> Self;

    /// Returns a pointer to the `static mut` variable behind this proxy
    fn get() -> *mut Self::Type;

    /// Consumes this singleton and returns a `&'static mut` reference to the variable behind it
    fn unwrap(self) -> &'static mut Self::Type
    where
        Self: Sized,
    {
        unsafe { &mut *Self::get() }
    }
}
