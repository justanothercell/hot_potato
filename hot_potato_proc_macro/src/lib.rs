#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, Punct};
use quote::{quote, ToTokens};
use syn::{parse, punctuated::Punctuated, parse::{Parse, ParseStream, ParseBuffer}, token::{Eq, Comma}, Expr, Token, parse_macro_input, Type};
use venial::{Function, TyExpr};


struct MagicValue {
    pub name: Ident,
    pub tk_colon: Token![:],
    pub ty: Type,
    pub tk_equals: Token![=],
    pub initializer: Expr
}

impl ToTokens for MagicValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.name.to_tokens(tokens);
        self.tk_colon.to_tokens(tokens);
        self.ty.to_tokens(tokens);
        self.tk_equals.to_tokens(tokens);
        self.initializer.to_tokens(tokens);
    }
}

impl Parse for MagicValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            tk_colon: input.parse()?,
            ty: input.parse()?,
            tk_equals: input.parse()?,
            initializer: input.parse()?
        })
    }
}

struct Magics(Vec<MagicValue>);

impl Parse for Magics {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let res = Punctuated::<MagicValue, Comma>::parse_terminated(input)?;
        Ok(Self(res.into_iter().collect()))
    }
}

#[proc_macro_attribute]
pub fn potato(attr: TokenStream, item: TokenStream) -> TokenStream {
    match venial::parse_declaration(proc_macro2::TokenStream::from(item)) {
        Ok(venial::Declaration::Function(fun)) => apply_to_fn(attr, fun),
        Ok(_other) => panic!("potato can only be applied to functions"),
        Err(_) => panic!("proc macro attribute could not read valid rust"),
    }
}

#[allow(unused_variables)]
fn apply_to_fn(attr: TokenStream, fun: Function) -> TokenStream {
    let magics = parse_macro_input!(attr as Magics);
    let magics_args = magics.0.iter().map(|m| {
        let ident = &m.name;
        let ty = &m.ty;
        quote!{ #ident: #ty }
    }).collect::<Vec<_>>();
    let magics_names = magics.0.iter().map(|m| &m.name).collect::<Vec<_>>();
    let magics_tys = magics.0.iter().map(|m| &m.ty).collect::<Vec<_>>();
    let magics_maker = magics.0.iter().map(|m| {
        let ident = &m.name;
        let ty = &m.ty;
        let expr = &m.initializer;
        quote!{ 
            let #ident: #ty = #expr;
            map.insert(stringify!(#ident), Box::new(#ident)); 
        }
    }).collect::<Vec<_>>();
    let magics_getter = magics.0.iter().map(|m| {
        let ident = &m.name;
        let ty = &m.ty;
        quote!{ 
            let #ident = potato.get::< #ty >(&stringify!(#ident)).clone();
        }
    }).collect::<Vec<_>>();

    let attrs = fun.attributes;
    let vis = fun.vis_marker;
    let qualifiers = fun.qualifiers;
    if qualifiers.extern_abi.is_some() || qualifiers.tk_async.is_some() || qualifiers.tk_const.is_some() ||
       qualifiers.tk_extern.is_some() || qualifiers.tk_unsafe.is_some() { panic!("potato may not be applied to functions with qualifiers") }
    let name = fun.name;
    let internal_name = Ident::new(&format!("{}__potato_internal", name.to_string()), Span::call_site());
    let generics = fun.generic_params;
    if generics.is_some() { panic!("potato may only be applied to functions without generics") }
    let args = fun.params;
    let arg_names = args.iter().map(|(param, _)| match param {
        venial::FnParam::Receiver(r) => panic!("potato may not be applied to self-methods"),
        venial::FnParam::Typed(t) => &t.name,
    }).collect::<Vec<&Ident>>();
    let arg_tys = args.iter().map(|(param, _)| match param {
        venial::FnParam::Receiver(_) => unreachable!(),
        venial::FnParam::Typed(t) => &t.ty,
    }).collect::<Vec<&TyExpr>>();
    let ret = fun.return_ty;
    let where_clause = fun.where_clause;
    let body = fun.body;
    if body.is_none() { panic!("potato only applicable to functions with a body") }

    TokenStream::from(quote! {
        #(#attrs)*
        #[no_mangle]
        #[export_name=concat!(module_path!(), "::", stringify!(#name), "__potato")]
        #[allow(non_snake_case)]
        pub fn #internal_name #generics ( #args, #(#magics_args,)* ) -> #ret #where_clause 
            #body
        
        #(#attrs)*
        #[allow(non_upper_case_globals)]
        #vis static #name: ::hot_potato::PotatoFunc<( #(#arg_tys,)*), ( #(#arg_tys,)* #(#magics_tys,)* ), #ret> = unsafe { ::hot_potato::PotatoFunc::new(
            concat!(module_path!(), "::", stringify!(#name), "__potato"),
            |mut writer| {
                let mut map: ::std::collections::HashMap<&'static str, Box<dyn ::std::any::Any>> = ::std::collections::HashMap::new();
                #( #magics_maker )*
                *writer = Some(map);
            },
            |( #(#arg_names,)* ), potato| {
                #( #magics_getter )*
                ( #(#arg_names,)* #(#magics_names,)* )
            }
        ) };

        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        ::hot_potato::submit!{
            unsafe { #name.handle() }
        }
    })
}