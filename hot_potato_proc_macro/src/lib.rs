#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use quote::quote;
use venial::{Function, TyExpr};

#[proc_macro_attribute]
pub fn potato(attr: TokenStream, item: TokenStream) -> TokenStream {
    match venial::parse_declaration(proc_macro2::TokenStream::from(item)) {
        Ok(venial::Declaration::Function(fun)) => apply_to_fn(attr, fun),
        Ok(_other) => panic!("potato can only be applied to functions"),
        Err(_) => panic!("proc macro attribute could not read valid rust"),
    }
}

#[allow(unused_variables)]
fn apply_to_fn(_attr: TokenStream, fun: Function) -> TokenStream {
    let attrs = fun.attributes;
    let vis = fun.vis_marker;
    let qualifiers = fun.qualifiers;
    let name = fun.name;
    let internal_name = Ident::new(&format!("{}__potato__internal", name.to_string()), Span::call_site());
    let generics = fun.generic_params;
    if generics.is_some() { panic!("potato may only be applied to functions without generics") }
    let args = fun.params;
    let arg_names = args.iter().map(|(param, _)| match param {
        venial::FnParam::Receiver(r) => panic!("potato may not be applied to self-methods"),
        venial::FnParam::Typed(t) => &t.name,
    }).collect::<Vec<&Ident>>();
    let arg_tys = args.iter().map(|(param, _)| match param {
        venial::FnParam::Receiver(_) => panic!("potato may not be applied to self-methods"),
        venial::FnParam::Typed(t) => &t.ty,
    }).collect::<Vec<&TyExpr>>();
    let ret = fun.return_ty;
    let where_clause = fun.where_clause;
    let body = fun.body;
    if body.is_none() { panic!("potato only applicable to functions with a body") }
    //panic!("{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}", attrs, vis, qualifiers, name, generics, args);
    TokenStream::from(quote! {
        #(#attrs)*
        #[export_name=concat!(module_path!(), "::", stringify!(#name), "__potato")]
        fn #internal_name #generics ( #args ) -> #ret #where_clause 
            #body
        
        #vis #qualifiers fn #name #generics ( #args ) -> #ret #where_clause {
            static mut potato: ::hot_potato::PotatoFunc<( #(#arg_tys,)* ), #ret> = ::hot_potato::PotatoFunc {
                func: None,
            };
        
            unsafe { potato ( #(#arg_names),* ) }
        }
    })
}