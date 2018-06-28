extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate exonum;

use proc_macro2::TokenStream;
#[proc_macro_derive(Schema)]
pub fn schema_custom_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();

    let ast: syn::DeriveInput = syn::parse2(input).unwrap();
    let ident = ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let functions = generate_functions(&ast.data);
    let output: TokenStream = quote!{
        #functions
    };

    output.into()
}


fn generate_functions(data: &syn::Data) -> TokenStream {
    match *data {
        syn::Data::Struct(ref data) => {
            match data.fields {
                syn::Fields::Named(ref fields) => {
                    for field in fields.named.iter() {
                        let ident = field.ident.clone().unwrap();
                        let field_type = field.ty.clone();
                        if let syn::Type::Path(type_path) = field_type {
                            return quote! {
                                pub struct #type_path {}
                            }
                        } else {
                            panic!("Panic");
                        }
                    }
                    quote!()
                },
                _ => panic!("Panic"),
            }
        },
        _ => panic!("Panic"),
    }
}

