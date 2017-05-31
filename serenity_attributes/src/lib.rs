#![feature(proc_macro)]
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use quote::Tokens;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn command(_: TokenStream, input: TokenStream) -> TokenStream {
    let ast = syn::parse_item(&input.to_string()).unwrap();
    impl_command(&ast).parse().unwrap()
}

// For organization purposes only.

mod utils;
mod parser_functions;

use utils::*;
use parser_functions::*;

fn impl_command(ast: &syn::Item) -> Tokens { 
    let command_name = to_pascal_case(ast.ident.as_ref());

    let mut settings = Settings::default();

    for attr in &ast.attrs {
        settings.parse_attr(&attr).unwrap();
    }

    let (declar, block) = match ast.node {
        syn::ItemKind::Fn(ref declar, .., ref block) => (declar.clone(), block.clone()),
        _ => panic!(r#"The "command" attribute can only be used on functions!"#),
    };

    // Not related to the `Settings`.
    let mut single_string_response = StringResponse::default();

    if let syn::FunctionRetTy::Ty(ref ty) = declar.output {
        if let syn::Ty::Path(_, ref pa) = *ty {
            match pa.segments[0].ident.clone().as_ref() {
                "&str" | "&'static str" | "String" => {
                    for s in &block.stmts {
                        if let syn::Stmt::Expr(ref expr) = *s {
                            if let syn::ExprKind::Lit(syn::Lit::Str(ref string_response, _)) = expr.node {
                                single_string_response = StringResponse(Some(string_response.clone()));
                            }
                        }
                    }
                },
                _ => unreachable!(),
            }
        }
    }
    let fn_args = parse_fn_args(&declar.inputs, *block);
    
    quote! {
        extern crate serenity as _serenity;
        #[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
        struct #command_name;
        impl _serenity::Command for #command_name {
            fn name(&self) -> String { format!("{}", #ast.ident) }
            #settings
            #fn_args
            #single_string_response
        }
    }
}
