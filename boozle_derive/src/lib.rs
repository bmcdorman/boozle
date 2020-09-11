use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Ident, ItemStruct, ItemTrait, TraitItem};

// use sha2::{Sha256, Digest};
// use bytes::{Buf, BufMut};

#[proc_macro_attribute]
pub fn interface(
  _attr: proc_macro::TokenStream,
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let mut original = proc_macro2::TokenStream::new();

  let input = parse_macro_input!(input as ItemTrait);

  /*let mut meta = Vec::new();

  for item in input.items.iter() {
      match item {
          TraitItem::Method(method) => {
              let mut hasher = Sha256::new();
              hasher.update(format!("{}", method.sig.ident).as_str().as_bytes());
              let bytes = hasher.finalize();
              let mut slice = bytes.as_slice();
              let id = slice.get_u64();
              vec.push((id, ))

              println!("{:?}", id);
          },
          _ => {}
      }
  }*/

  input.to_tokens(&mut original);
  let name = input.ident;
  let metadata_name = syn::parse_str::<Ident>(format!("{}{}", name, "_meta").as_str()).unwrap();
  println!("{:?}", metadata_name);
  // let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
  let expanded = quote! {
      #original

      const #metadata_name: u32 = 0;
  };

  // Hand the output tokens back to the compiler.
  proc_macro::TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn class(
  attr: proc_macro::TokenStream,
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let mut original = proc_macro2::TokenStream::new();
  let interface = parse_macro_input!(attr as Ident);

  let input = parse_macro_input!(input as ItemStruct);

  input.to_tokens(&mut original);
  let name = input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
  let expanded = quote! {
      #original

      // The generated impl.
      impl #impl_generics actix::Actor for #name #ty_generics #where_clause {
          type Context = actix::Context<Self>;
      }

      impl #impl_generics actix::Handler<boozle::call::Call> for #name #ty_generics #where_clause {
          type Result = Result<boozle::call::Return, boozle::call::Error>;

          fn handle(&mut self, msg: boozle::call::Call, _: &mut actix::Context<Self>) -> Self::Result {
              Ok(boozle::call::Return { result: msg.argument })
          }
      }
  };

  // Hand the output tokens back to the compiler.
  proc_macro::TokenStream::from(expanded)
}
