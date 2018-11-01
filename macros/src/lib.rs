extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate rand;
extern crate syn;

use proc_macro::TokenStream;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use proc_macro2::Span;
use quote::quote;
use rand::{Rng, SeedableRng};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, ItemStatic, Token,
};

/// Attribute to declare an owned singleton
///
/// This attribute must be applied to a `static [mut]` variable.
///
/// The attribute accepts two arguments: `Send` and `Sync` (e.g. `#[Singleton(Send, Sync)]`)
///
/// The expansion will produce a proxy struct whose name matches the identifier of the `static`
/// variable.
///
/// For more information read the crate level documentation of the `owned-singleton` crate.
#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn Singleton(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStatic);
    let args = parse_macro_input!(args as Args);

    if let Err(e) = check(&item) {
        return e.to_compile_error().into();
    }

    let attrs = &item.attrs;
    let vis = &item.vis;
    let ident = &item.ident;
    let ty = &item.ty;
    let expr = &item.expr;
    let alias = mk_ident();

    let mut items = vec![];
    let symbol = format!("{}::{}", ident, alias);
    items.push(quote!(
        #(#attrs)*
        #[export_name = #symbol]
        static mut #alias: #ty = #expr;

        #vis struct #ident { #alias: owned_singleton::export::NotSendOrSync }

        unsafe impl owned_singleton::Singleton for #ident {
            type Type = #ty;

            #[inline]
            unsafe fn new() -> Self {
                #ident { #alias: owned_singleton::export::PhantomData }
            }

            #[inline]
            fn get() -> *mut Self::Type {
                unsafe { &mut #alias }
            }
        }

        impl owned_singleton::export::Deref for #ident {
            type Target = #ty;

            #[inline]
            fn deref(&self) -> &Self::Target {
                unsafe { &#alias }
            }
        }

        unsafe impl owned_singleton::export::StableDeref for #ident {}
    ));

    if args.send {
        items.push(quote!(
            unsafe impl Send for #ident where #ty: Send {}
        ));
    }

    if args.sync {
        items.push(quote!(
            unsafe impl Sync for #ident where #ty: Sync {}
        ));
    }

    if item.mutability.is_some() {
        items.push(quote!(
            impl owned_singleton::export::DerefMut for #ident {
                #[inline]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut #alias }
                }
            }
        ));
    }

    quote!(#(#items)*).into()
}

struct Args {
    send: bool,
    sync: bool,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut send = false;
        let mut sync = false;
        let punctuated = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;

        for ident in punctuated {
            match &*ident.to_string() {
                "Send" => {
                    if send {
                        return Err(parse::Error::new(ident.span(), "this trait appears twice"));
                    }

                    send = true;
                }
                "Sync" => {
                    if sync {
                        return Err(parse::Error::new(ident.span(), "this trait appears twice"));
                    }

                    sync = true;
                }
                _ => {
                    return Err(parse::Error::new(
                        ident.span(),
                        "expected one of: Send or Sync",
                    ))
                }
            }
        }

        Ok(Args { send, sync })
    }
}

fn check(_item: &ItemStatic) -> parse::Result<()> {
    // TODO

    Ok(())
}

fn mk_ident() -> Ident {
    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let secs = elapsed.as_secs();
    let nanos = elapsed.subsec_nanos();

    let count = CALL_COUNT.fetch_add(1, Ordering::SeqCst) as u32;
    let mut seed: [u8; 16] = [0; 16];

    for (i, v) in seed.iter_mut().take(8).enumerate() {
        *v = ((secs >> (i * 8)) & 0xFF) as u8
    }

    for (i, v) in seed.iter_mut().skip(8).take(4).enumerate() {
        *v = ((nanos >> (i * 8)) & 0xFF) as u8
    }

    for (i, v) in seed.iter_mut().skip(12).enumerate() {
        *v = ((count >> (i * 8)) & 0xFF) as u8
    }

    let mut rng = rand::rngs::SmallRng::from_seed(seed);
    Ident::new(
        &(0..16)
            .map(|i| {
                if i == 0 || rng.gen() {
                    ('a' as u8 + rng.gen::<u8>() % 25) as char
                } else {
                    ('0' as u8 + rng.gen::<u8>() % 10) as char
                }
            }).collect::<String>(),
        Span::call_site(),
    )
}
