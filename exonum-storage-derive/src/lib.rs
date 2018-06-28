#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate exonum;
use exonum::storage::MapIndex;


use proc_macro2::TokenStream;

#[proc_macro_derive(Schema, attributes(schema))]
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
                        let attributes = field.attrs.clone();
                        let mut index_type = None;
                        for attribute in &attributes {
                            let meta = attribute.interpret_meta();
                            if let Some(syn::Meta::List(ref list)) = meta {
                                if list.ident == "schema" {
                                    if let syn::NestedMeta::Meta(syn::Meta::Word(ref word)) = list.nested.first().unwrap().into_value() {
                                        index_type = Some(word.clone());
                                    } else {
                                        panic!("Error3")
                                    }
                                } else {
                                    panic!("Error2")
                                }
                            } else {
                                panic!("Error");
                            }
                        }
                        let index_type = match index_type {
                            Some(x) => x,
                            None => continue,
                        };
                        if let syn::Type::Path(ref type_path) = field_type {
                            let last_segment = type_path.path.segments.last().map(|v| v.into_value()).clone().unwrap();
                            let index_type_name = last_segment.ident.clone();
                            let (key_type, value_type) = match get_type_parameters(last_segment) {
                                Some(v) => v,
                                None => continue,
                            };
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

                            let read_func_ident = ident.to_string() + "_read";
                            let read_func_ident = syn::Ident::new(&read_func_ident, proc_macro2::Span::call_site());

                            let impl_read = quote!{
                                impl<'a, T: AsRef<::exonum::storage::Snapshot>> #index_type_name <T, #key_type, #value_type> {
                                    pub fn read(&self, view: &'a ::exonum::storage::Snapshot) -> #index_type<&'a ::exonum::storage::Snapshot, #key_type, #value_type> {
                                        #index_type::new(stringify!(ident), view)
                                    }
                                }

                                impl<T: AsRef<::exonum::storage::Snapshot>> #struct_ident <T> {
                                    pub fn #read_func_ident(&self) -> #index_type<& ::exonum::storage::Snapshot, #key_type, #value_type> {
                                        self.#ident.read(self.view.as_ref())
                                    }
                                }
                            };

                            let write_func_ident = ident.to_string() + "_write";
                            let write_func_ident = syn::Ident::new(&write_func_ident, proc_macro2::Span::call_site());

                            let impl_write = quote!{
                                impl<'a> #index_type_name <&'a mut ::exonum::storage::Fork, #key_type, #value_type> {
                                    pub fn write(&self, view: &'a mut ::exonum::storage::Fork) -> #index_type<&'a mut ::exonum::storage::Fork, #key_type, #value_type> {
                                        #index_type::new(stringify!(ident), view)
                                    }
                                }

                                impl<'a> #struct_ident <&'a mut ::exonum::storage::Fork> {
                                    pub fn #write_func_ident(&'a mut self) -> #index_type<&'a mut ::exonum::storage::Fork, #key_type, #value_type> {
                                        #index_type::new(stringify!(ident), self.view)
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
                        } else {}
                    }
                    tokens.extend(
                        quote!{
                            impl<T> #struct_ident <T> {
                                pub fn new(view: T) -> Self {
                                    Self {
                                        view,
                                        #struct_fields
                                    }
                                }
                            }
                        }
                    );
                    tokens
                },
                _ => panic!("Panic1"),
            }
        },
        _ => panic!("Panic2"),
    }
}

fn get_type_parameters(last_segment: &syn::PathSegment) -> Option<(syn::GenericArgument, syn::GenericArgument)> {
    let types = if let syn::PathArguments::AngleBracketed(ref args) = last_segment.arguments {
        let mut args = args.args.iter();
        let _ = args.next();
        let key = args.next().unwrap();
        let value = args.next().unwrap();
        Some((key.clone(), value.clone()))
    } else {
        None
    };
    types
}

