#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate exonum;

use proc_macro2::TokenStream;

#[derive(Clone, Debug, Copy)]
enum IndexMutability {
    Mutable,
    Immutable,
}

impl IndexMutability {
    pub fn into_token_stream(self) -> TokenStream {
        match self {
            IndexMutability::Immutable => quote!(&'a ::exonum::storage::Snapshot),
            IndexMutability::Mutable => quote!(&'a mut ::exonum::storage::Fork),
        }
    }
}

#[derive(Clone, Debug, Copy)]
enum IndexTypeParamsCount {
    Zero,
    One,
    Two,
}

impl From<IndexType> for IndexTypeParamsCount {
    fn from(index_type: IndexType) -> Self {
        match index_type {
            IndexType::Map => IndexTypeParamsCount::Two,
            IndexType::List => IndexTypeParamsCount::One,
            IndexType::ProofMap => IndexTypeParamsCount::Two,
            IndexType::ProofList => IndexTypeParamsCount::One,
            IndexType::Entry => IndexTypeParamsCount::Zero,
            IndexType::SparseList => IndexTypeParamsCount::One,
            IndexType::KeySet => IndexTypeParamsCount::One,
            IndexType::ValueSet => IndexTypeParamsCount::One,
        }
    }
}

#[derive(Clone, Debug, Copy)]
enum IndexType {
    Map,
    List,
    ProofMap,
    ProofList,
    Entry,
    SparseList,
    KeySet,
    ValueSet,
}

impl From<syn::Ident> for IndexType {
    fn from(ident: syn::Ident) -> Self {
        match ident {
            ref x if x == "MapIndex" => IndexType::Map,
            ref x if x == "MapIndex" => IndexType::List,
            ref x if x == "MapIndex" => IndexType::ProofMap,
            ref x if x == "MapIndex" => IndexType::ProofList,
            ref x if x == "MapIndex" => IndexType::Entry,
            ref x if x == "MapIndex" => IndexType::SparseList,
            ref x if x == "KeySet" => IndexType::KeySet,
            ref x if x == "ValueSet" => IndexType::ValueSet,
            _ => panic!("Invalid index type"),
        }
    }
}

impl IndexType {
    fn into_token_stream(
        self,
        mutability: IndexMutability,
        generic_params: &syn::PathSegment,
    ) -> TokenStream {
        let index_type = match self {
            IndexType::Map => quote!(::exonum::storage::MapIndex),
            IndexType::List => quote!(::exonum::storage::ListIndex),
            IndexType::ProofMap => quote!(::exonum::storage::ProofMapIndex),
            IndexType::ProofList => quote!(::exonum::storage::ProofListIndex),
            IndexType::Entry => quote!(::exonum::storage::Entry),
            IndexType::SparseList => quote!(::exonum::storage::SparseListIndex),
            IndexType::KeySet => quote!(::exonum::storage::KeySetIndex),
            IndexType::ValueSet => quote!(::exonum::storage::ValueSetIndex),
        };

        let view_type = mutability.into_token_stream();

        type_with_parameters(index_type, view_type, self.into(), generic_params)
    }
}

fn type_with_parameters(
    ty: TokenStream,
    first_param: TokenStream,
    param_count: IndexTypeParamsCount,
    generic_params: &syn::PathSegment,
) -> TokenStream {
    use IndexTypeParamsCount::*;
    match param_count {
        Zero => quote!(#ty <#first_param>),
        One => {
            let param = get_type_parameter(generic_params).unwrap();
            quote!(#ty <#first_param, #param>)
        }
        Two => {
            let (second, third) = get_type_parameters(generic_params).unwrap();
            quote!(#ty <#first_param, #second, #third>)
        }
    }
}

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
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let mut tokens = TokenStream::new();
                let mut struct_fields: syn::punctuated::Punctuated<
                    _,
                    syn::token::Comma,
                > = syn::punctuated::Punctuated::new();
                for field in fields.named.iter() {
                    let ident = field.ident.clone().unwrap();
                    let field_type = field.ty.clone();
                    let attributes = field.attrs.clone();
                    let mut index_type = None;
                    for attribute in &attributes {
                        let meta = attribute.interpret_meta();
                        if let Some(syn::Meta::List(ref list)) = meta {
                            if list.ident == "schema" {
                                if let syn::NestedMeta::Meta(syn::Meta::Word(ref word)) =
                                    list.nested.first().unwrap().into_value()
                                {
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
                    let index_type_raw = match index_type {
                        Some(x) => x,
                        None => continue,
                    };
                    let index_type: IndexType = index_type_raw.clone().into();
                    if let syn::Type::Path(ref type_path) = field_type {
                        let last_segment = type_path
                            .path
                            .segments
                            .last()
                            .map(|v| v.into_value())
                            .clone()
                            .unwrap();
                        let user_index_type_name = last_segment.ident.clone();

                        let index_type_token_stream =
                            index_type.into_token_stream(IndexMutability::Immutable, &last_segment);

                        tokens.extend(index_type_definition(&user_index_type_name));

                        let user_index_type_token_stream = type_with_parameters(
                            quote!(#user_index_type_name),
                            quote!(T),
                            index_type.into(),
                            &last_segment,
                        );

                        tokens.extend(impl_read_methods(
                            &ident,
                            &index_type_raw,
                            &struct_ident,
                            &user_index_type_token_stream,
                            &index_type_token_stream,
                        ));

                        let index_type_token_stream =
                            index_type.into_token_stream(IndexMutability::Mutable, &last_segment);

                        let user_index_type_token_stream = type_with_parameters(
                            quote!(#user_index_type_name),
                            quote!(&'a mut ::exonum::storage::Fork),
                            index_type.into(),
                            &last_segment,
                        );

                        tokens.extend(impl_write_methods(
                            &ident,
                            &index_type_raw,
                            &struct_ident,
                            &user_index_type_token_stream,
                            &index_type_token_stream,
                        ));

                        struct_fields.push_value(quote!(#ident : {
                                    let index: #field_type = #user_index_type_name::new();
                                    index
                                }));
                    } else {
                    }
                }
                tokens.extend(quote!{
                    impl<T> #struct_ident <T> {
                        pub fn new(view: T) -> Self {
                            Self {
                                view,
                                #struct_fields
                            }
                        }
                    }
                });
                tokens
            }
            _ => panic!("Panic1"),
        },
        _ => panic!("Panic2"),
    }
}

fn index_type_definition(index_type_name: &syn::Ident) -> TokenStream {
    quote! {
        pub struct #index_type_name <T, K, V> {
            _i: ::std::marker::PhantomData<T>,
            _k: ::std::marker::PhantomData<K>,
            _v: ::std::marker::PhantomData<V>,
        }

        impl<T, K, V> #index_type_name <T, K, V> {
            pub fn new() -> Self {
                Self {
                    _i: Default::default(),
                    _k: Default::default(),
                    _v: Default::default(),
                }
            }
        }
    }
}

fn impl_read_methods(
    ident: &syn::Ident,
    index_type_raw: &syn::Ident,
    struct_ident: &syn::Ident,
    user_index_type_token_stream: &TokenStream,
    index_type_token_stream: &TokenStream,
) -> TokenStream {
    let read_func_ident = func_ident(ident.clone(), IndexMutability::Immutable);
    quote!{
        impl<'a, T: AsRef<::exonum::storage::Snapshot>> #user_index_type_token_stream {
            pub fn read(&self, view: &'a ::exonum::storage::Snapshot) -> #index_type_token_stream {
                #index_type_raw::new(stringify!(ident), view)
            }
        }

        impl<'a, T: AsRef<::exonum::storage::Snapshot>> #struct_ident <T> {
            pub fn #read_func_ident(&'a self) -> #index_type_token_stream {
                self.#ident.read(self.view.as_ref())
            }
        }
    }
}

fn impl_write_methods(
    ident: &syn::Ident,
    index_type_raw: &syn::Ident,
    struct_ident: &syn::Ident,
    user_index_type_token_stream: &TokenStream,
    index_type_token_stream: &TokenStream,
) -> TokenStream {
    let write_func_ident = func_ident(ident.clone(), IndexMutability::Mutable);
    quote!{
        impl<'a> #user_index_type_token_stream {
            pub fn write(&self, view: &'a mut ::exonum::storage::Fork) -> #index_type_token_stream {
                #index_type_raw::new(stringify!(ident), view)
            }
        }

        impl<'a> #struct_ident <&'a mut ::exonum::storage::Fork> {
            pub fn #write_func_ident(&'a mut self) -> #index_type_token_stream {
                self.#ident.write(self.view)
            }
        }
    }
}

fn get_type_parameters(
    last_segment: &syn::PathSegment,
) -> Option<(syn::GenericArgument, syn::GenericArgument)> {
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

fn get_type_parameter(last_segment: &syn::PathSegment) -> Option<syn::GenericArgument> {
    let type_parameter =
        if let syn::PathArguments::AngleBracketed(ref args) = last_segment.arguments {
            let mut args = args.args.iter();
            let _ = args.next();
            let value = args.next().unwrap();
            Some(value.clone())
        } else {
            None
        };
    type_parameter
}

fn func_ident(ident: syn::Ident, mutability: IndexMutability) -> syn::Ident {
    let postfix = match mutability {
        IndexMutability::Immutable => "_read",
        IndexMutability::Mutable => "_write",
    };
    let read_func_ident = ident.to_string() + postfix;
    syn::Ident::new(&read_func_ident, proc_macro2::Span::call_site())
}
