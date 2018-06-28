#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate exonum;
use exonum::storage::MapIndex;


use proc_macro2::TokenStream;

#[proc_macro_derive(Schema)]
pub fn schema_custom_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();

    let ast: syn::DeriveInput = syn::parse2(input).unwrap();

    let functions = generate_functions(&ast);
    let output: TokenStream = quote!{
        #functions
    };

    output.into()
}


fn generate_functions(ast: &syn::DeriveInput) -> TokenStream {
    let struct_ident = ast.ident.clone();

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let data = &ast.data;

    match *data {
        syn::Data::Struct(ref data) => {
            match data.fields {
                syn::Fields::Named(ref fields) => {
                    let mut tokens = TokenStream::new();
                    let mut struct_fields: syn::punctuated::Punctuated<_, syn::token::Comma> = syn::punctuated::Punctuated::new();
                    for field in fields.named.iter() {
                        let ident = field.ident.clone().unwrap();
                        let field_type = field.ty.clone();
                        if let syn::Type::Path(ref type_path) = field_type {
                            let last_segment = type_path.path.segments.last().map(|v| v.into_value()).clone().unwrap();
                            let index_type_name = last_segment.ident.clone();
                            let (key_type, value_type) = get_type_parameters(last_segment);
                            let struct_definition = quote!{
                                pub struct #index_type_name <T, K, V> {
                                    _i: ::std::marker::PhantomData<T>,
                                    _k: ::std::marker::PhantomData<K>,
                                    _v: ::std::marker::PhantomData<V>,
                                }
                            };
                            let impl_new = quote!{
                                impl<T, K, V> #index_type_name <T, K, V> {
                                    pub fn new() -> Self {
                                        Self {
                                            _i: Default::default(),
                                            _k: Default::default(),
                                            _v: Default::default(),
                                        }
                                    }
                                }
                            };
                            let impl_read = quote!{
                                impl<T: AsRef<::exonum::storage::Snapshot>> #index_type_name <T, #key_type, #value_type> {
                                    pub fn read(&self, view: T) -> MapIndex<T, #key_type, #value_type> {
                                        MapIndex::new(stringify!(ident), view)
                                    }
                                }
                            };
                            let impl_write = quote!{
                                impl<'a> #index_type_name <&'a mut ::exonum::storage::Fork, #key_type, #value_type> {
                                    pub fn write(&self, view: &'a mut ::exonum::storage::Fork) -> MapIndex<&'a mut ::exonum::storage::Fork, #key_type, #value_type> {
                                        MapIndex::new(stringify!(ident), view)
                                    }
                                }
                            };
                            tokens.extend(quote! {
                                #struct_definition
                                #impl_new
                                #impl_read
                                #impl_write
                            });
                            struct_fields.push_value(
                                quote!(#ident : {
                                    let index: #field_type = #index_type_name::new();
                                    index
                                })
                            );
                        } else {
                            panic!("Panic");
                        }
                    }
                    tokens.extend(
                        quote!{
                            impl<T> #struct_ident <T> {
                                pub fn new() -> Self {
                                    Self {
                                        #struct_fields
                                    }
                                }
                            }
                        }
                    );
                    tokens
                },
                _ => panic!("Panic"),
            }
        },
        _ => panic!("Panic"),
    }
}

fn get_type_parameters(last_segment: &syn::PathSegment) -> (syn::GenericArgument, syn::GenericArgument) {
    let types = if let syn::PathArguments::AngleBracketed(ref args) = last_segment.arguments {
        let mut args = args.args.iter();
        let _ = args.next();
        let key = args.next().unwrap();
        let value = args.next().unwrap();
        (key.clone(), value.clone())
    } else {
        panic!("Panic");
    };
    types
}

