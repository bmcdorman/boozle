use boozle_parser::{Decl, Fn, Mod, Param, Path, Svc, Trait, Type, Unit};
use digest::Digest;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use sha2::Sha256;
use std::collections::HashMap;
use std::convert::TryInto;
use syn;

fn generate_mod(module: &Mod) -> TokenStream {
  let ident = syn::Ident::new(module.name.text.as_str(), Span::call_site());
  match &module.decls {
    Some(decls) => {
      let mut children = TokenStream::new();
      for decl in decls.iter() {
        children.extend(generate_decl(decl));
      }
      quote! {
        mod #ident {
          #children
        }
      }
    }
    None => quote! { mod #ident; },
  }
}

fn id(text: &str) -> u64 {
  let digest = Sha256::digest(text.as_bytes());
  let (top, _) = digest.split_at(std::mem::size_of::<u64>());
  u64::from_ne_bytes(top.try_into().unwrap())
}

fn generate_svc(svc: &Svc) -> TokenStream {
  let ident = syn::Ident::new(svc.name.text.as_str(), Span::call_site());
  let id = id(svc.name.text.as_str());
  quote! {
    u64 #ident = #id;
  }
}

fn generate_path(path: &Path) -> TokenStream {
  let components: Vec<syn::Ident> = path
    .components
    .iter()
    .map(|id| syn::Ident::new(id.text.as_str(), Span::call_site()))
    .collect();
  quote! {
    #(#components)::*
  }
}

fn generate_type(ty: &Type) -> TokenStream {
  let path = generate_path(&ty.path);
  quote! {
    #path
  }
}

fn generate_param(param: &Param) -> TokenStream {
  let ident = syn::Ident::new(param.name.text.as_str(), Span::call_site());
  let ty = generate_type(&param.ty);
  quote! {
    #ident : #ty
  }
}

fn generate_fn(fn_: &Fn) -> TokenStream {
  let ident = syn::Ident::new(fn_.name.text.as_str(), Span::call_site());
  let params: Vec<TokenStream> = fn_.params.iter().map(generate_param).collect();
  quote! {
    fn #ident(#(#params),*);
  }
}

fn generate_trait(trait_: &Trait) -> TokenStream {
  let ident = syn::Ident::new(trait_.name.text.as_str(), Span::call_site());
  let members: Vec<TokenStream> = trait_.members.iter().map(generate_fn).collect();
  quote! {
    trait #ident {
      #(#members)*
    }
  }
}

fn generate_decl(decl: &Decl) -> TokenStream {
  match decl {
    Decl::Mod(module) => generate_mod(&module),
    Decl::Svc(svc) => generate_svc(&svc),
    Decl::Trait(trait_) => generate_trait(&trait_),
    _ => TokenStream::new(),
  }
}

fn generate_unit(unit: &Unit) -> TokenStream {
  let mut stream = TokenStream::new();
  for decl in unit.decls.iter() {
    stream.extend(generate_decl(&decl));
  }
  stream
}
