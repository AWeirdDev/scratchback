use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::{ Ident, TokenTree };

use venial::{ parse_item, Error, Fields, GenericArg, Item, TypeExpr };
use quote::quote;

macro_rules! ok_or_rt {
    ($e:expr) => {
        match $e {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        }
    };
}

fn derive(input: TokenStream) -> Result<TokenStream, Error> {
    let item = ok_or_rt!(parse_item(input.into()));

    match item {
        Item::Struct(st) => {
            let Fields::Named(fields) = st.fields else {
                return Err(Error::new_at_span(st.fields.span(), "Expected named fields"));
            };

            let mut flattens_to: Option<String> = None;
            let mut has_id = false;
            let mut map: HashMap<u8, Ident> = HashMap::new();

            for field in fields.fields.items() {
                let field_name = field.name.to_string();

                for attr in &field.attributes {
                    let name = &attr.path.last().unwrap().to_string();
                    if name != "id" {
                        continue;
                    }

                    let Some(value_tokens) = attr.get_value_tokens().first() else {
                        return Err(
                            Error::new_at_span(attr.span(), "Expected #[id(...)], got no value")
                        );
                    };

                    has_id = true;
                    match value_tokens {
                        TokenTree::Literal(lit) => {
                            let Ok(id) = lit.to_string().parse::<u8>() else {
                                return Err(
                                    Error::new_at_span(
                                        lit.span(),
                                        "Cannot parse into u8 (range: 0-255)"
                                    )
                                );
                            };

                            if map.contains_key(&id) {
                                return Err(
                                    Error::new_at_span(lit.span(), "This id already exists")
                                );
                            }

                            map.insert(id, field.name.clone());
                        }
                        TokenTree::Ident(ident) => {
                            let identifier = &ident.to_string();
                            if identifier != "flatten" {
                                return Err(
                                    Error::new_at_span(
                                        ident.span(),
                                        "Expected either a numeric literal (u8) or `flatten`:\n#[id(1)]\n#[id(flatten)]"
                                    )
                                );
                            }

                            let path = field.ty.as_path().unwrap();
                            let last_typ = path.segments.last().unwrap();
                            if &last_typ.ident.to_string() != "Vec" {
                                return Err(
                                    Error::new_at_span(
                                        field.ty.span(),
                                        "Expected Vec<String> for #[id(flatten)]"
                                    )
                                );
                            }
                            let generic_args = last_typ.generic_args.as_ref().unwrap();
                            let first_t = generic_args.args.first().unwrap();
                            let GenericArg::TypeOrConst { expr } = &first_t.0 else {
                                return Err(
                                    Error::new_at_span(
                                        field.ty.span(),
                                        "Expected Vec<String> for #[id(flatten)]"
                                    )
                                );
                            };
                            let TokenTree::Ident(vec_t_token) = expr.tokens.first().unwrap() else {
                                return Err(
                                    Error::new_at_span(
                                        field.ty.span(),
                                        "Expected Vec<String> for #[id(flatten)]"
                                    )
                                );
                            };

                            if &vec_t_token.to_string() != "String" {
                                return Err(
                                    Error::new_at_span(field.ty.span(), "Expected Vec<String>")
                                );
                            }

                            if flattens_to.is_some() {
                                return Err(
                                    Error::new_at_span(
                                        ident.span(),
                                        "Can only have one flattened item at the end"
                                    )
                                );
                            }

                            flattens_to = Some(field_name.clone());
                        }
                        _ => {
                            return Err(
                                Error::new_at_span(
                                    value_tokens.span(),
                                    "Expected either a numeric literal (u8) or `flatten`:\n#[id(1)]\n#[id(flatten)]"
                                )
                            );
                        }
                    }
                }

                if !has_id {
                    return Err(Error::new_at_span(field.span(), "Assign an ID: #[id(...)]"));
                }
            }

            let name = st.name;
            let mut field_names = Vec::new();
            let mapped_de_items = map.iter().map(|(k, v)| {
                field_names.push(v);
                quote! {
                    let #v = std::mem::replace(&mut arr[#k as usize], String::new()).sb_string_to()?;
                }
            });
            let items_n = mapped_de_items.len();

            let result =
                quote! {
                impl #name {
                    /// Create a new instance of this struct from a `scratchback`-encoded string.
                    fn from_sb_encoded(numbers: &str) -> Option<Self> {
                        use ::scratchback::encoding::{ SbStringTo, Encoding };
                        let mut arr = Encoding::decode_items_to_array::<#items_n>(numbers)?;
                        #( #mapped_de_items )*

                        Some(Self {
                            #(#field_names, )*
                        })
                    }
                    /// Serialize this struct instance to a `scratchback`-encoded string.
                    fn sb_encode(self) -> Option<String> {
                        use ::scratchback::encoding::{ SbToString, Encoding };
                        
                        let Self { #(#field_names, )* } = self;
                        #(
                            let s = #field_names.sb_to_string();
                            let #field_names = s.as_str();
                        )*
                        let d = [#(#field_names, )*];
                        Encoding::encode_items(&d)
                    }
                }
            };
            Ok(result.into())
        }

        Item::Enum(en) => {
            let fields = en.variants;
            let mut map: HashMap<u8, (Ident, TypeExpr)> = HashMap::new();

            for field in fields.items() {
                for attr in &field.attributes {
                    let attr_name = &attr.path.last().as_ref().unwrap().to_string();
                    if attr_name != "id" {
                        continue;
                    }

                    let Some(value_token) = attr.get_value_tokens().first() else {
                        return Err(Error::new_at_span(attr.span(), "Assign an ID: #[id(...)]"));
                    };

                    let Ok(id) = value_token.to_string().parse::<u8>() else {
                        return Err(
                            Error::new_at_span(attr.span(), "Cannot parse into u8 (range: 0-255)")
                        );
                    };

                    if map.contains_key(&id) {
                        return Err(
                            Error::new_at_span(value_token.span(), "This id already exists")
                        );
                    }

                    let Fields::Tuple(t) = &field.fields else {
                        return Err(
                            Error::new_at_span(field.fields.span(), "Expected tuple-based fields")
                        );
                    };
                    let tuple_fields = &t.fields.inner;
                    if tuple_fields.len() > 1 {
                        return Err(
                            Error::new_at_span(
                                field.fields.span(),
                                "Currently, you can only have one item for a tuple field"
                            )
                        );
                    }
                    let (tuple_field, _) = tuple_fields.first().unwrap();
                    map.insert(id, (field.name.clone(), tuple_field.ty.clone()));
                }
            }

            let name = en.name;
            let mut mapped_en_items = Vec::new();
            let mapped_de_items = map.iter().map(|(k, (variant, typ))| {
                let k = k.to_string();

                mapped_en_items.push(
                    quote! {
                        #k => Self::#variant(#typ::from_sb_encoded(&x)?),
                    }
                );

                quote! {
                    Self::#variant(x) => (x, #k), 
                }
            });

            let result =
                quote! {
                impl #name {
                    /// Serialize this enum instance to a `scratchback`-encoded string.
                    fn sb_encode(self) -> Option<String> {
                        use ::scratchback::encoding::{ Encoding, EncodingTable };

                        let (st, id) = match self {
                            #(#mapped_de_items)*
                        };

                        Some(format!("{}{}{}", Encoding::encode(id)?, EncodingTable::encode(Encoding::SPLITTER)?, st.sb_encode()?))
                    }

                    /// Create a new instance of this enum from a `scratchback`-encoded string.
                    fn from_sb_encoded(numbers: &str) -> Option<Self> {
                        use ::scratchback::encoding::{ Encoding };

                        let n = Encoding::decode(&numbers[0..2])?;
                        let x = &numbers[2..numbers.len()];

                        let res = match n.as_ref() {
                            #(#mapped_en_items)*
                            _ => {
                                return None;
                            }
                        };
                        Some(res)
                    }
                }
            };

            Ok(result.into())
        }

        x => Err(Error::new_at_span(x.span(), "Not supported.")),
    }
}

/// Marks a `struct` as a Scratch object.
///
/// Zero-indexed.
///
/// ```no_run
/// #[derive(ScratchObject)]
/// struct Player {
///     #[id(0)]
///     name: String,
/// }
///
/// Player::from_sb_encoded("...");
/// ```
#[proc_macro_derive(ScratchObject, attributes(id))]
pub fn derive_scratch(input: TokenStream) -> TokenStream {
    let res = derive(input);
    res.unwrap_or_else(|err| err.to_compile_error().into()).into()
}
