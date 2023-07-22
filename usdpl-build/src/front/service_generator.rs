use std::collections::HashSet;

use nrpc_build::IServiceGenerator;
use prost_build::Service;
use prost_types::{DescriptorProto, EnumDescriptorProto, FieldDescriptorProto, FileDescriptorSet};

use super::SharedState;

pub struct WasmServiceGenerator {
    shared: SharedState,
}

impl WasmServiceGenerator {
    pub fn with_state(state: &SharedState) -> Self {
        Self {
            shared: state.clone(),
        }
    }
}

fn generate_service_methods(
    service: &Service,
    fds: &FileDescriptorSet,
) -> proc_macro2::TokenStream {
    let mut gen_methods = Vec::with_capacity(service.methods.len());
    for method in &service.methods {
        let method_name_str = method.name.clone();
        let method_name = quote::format_ident!("{}", method.name);
        let method_input = quote::format_ident!("{}{}", &service.name, method.input_type);
        let method_output = quote::format_ident!("{}{}Wasm", &service.name, method.output_type);
        let method_output_as_in = quote::format_ident!("{}{}", &service.name, method.output_type);

        let input_type = find_message_type(&method.input_type, &service.package, fds)
            .expect("Protobuf message is used but not found");

        let mut input_params = Vec::with_capacity(input_type.field.len());
        let mut params_to_fields = Vec::with_capacity(input_type.field.len());

        match (method.client_streaming, method.server_streaming) {
            (false, false) => {
                for field in &input_type.field {
                    //let param_name = quote::format_ident!("val{}", i.to_string());
                    let type_enum = ProtobufType::from_field(field, &service.name, false);
                    //let rs_type_name = type_enum.to_tokens();
                    let js_type_name = type_enum.to_wasm_tokens();
                    let rs_type_name = type_enum.to_tokens();
                    let field_name = quote::format_ident!(
                        "{}",
                        field
                            .name
                            .as_ref()
                            .expect("Protobuf message field needs a name")
                    );
                    input_params.push(quote::quote! {
                        #field_name: #js_type_name,
                    });
                    params_to_fields.push(quote::quote! {
                        #field_name: #rs_type_name::from_wasm(#field_name.into()),//: #field_name,
                    });
                }
                let params_to_fields_transformer = if input_type.field.len() == 1 {
                    let field_name = quote::format_ident!(
                        "{}",
                        input_type.field[0]
                            .name
                            .as_ref()
                            .expect("Protobuf message field needs a name")
                    );
                    quote::quote! {
                        let val = #method_input::from_wasm(#field_name.into());
                    }
                } else if input_type.field.is_empty() {
                    quote::quote! {
                        let val = #method_input {};
                    }
                } else {
                    quote::quote! {
                        let val = #method_input {
                            #(#params_to_fields)*
                        };
                    }
                };

                gen_methods.push(quote::quote! {
                    #[wasm_bindgen]
                    pub async fn #method_name(&self, #(#input_params)*) -> Option<#method_output> {

                        #params_to_fields_transformer

                        match self.service.#method_name(val.into()).await {
                            Ok(x) => {
                                let x2: #method_output_as_in = x.into();
                                Some(x2.into_wasm())
                            },
                            Err(e) => {
                                // log error
                                log::error!("service:{}|method:{}|error:{}", self.service.descriptor(), #method_name_str, e);
                                None
                            }
                        }
                    }
                });
            },
            (true, false) => {
                // many -> 1
                gen_methods.push(quote::quote! {
                    #[wasm_bindgen]
                    pub async fn #method_name(&self, generator: js_sys::Function) -> Option<#method_output> {

                        // function into Rust futures Stream
                        let stream = Box::new(::usdpl_front::wasm::JsFunctionStream::<#method_input>::from_function(generator));

                        match self.service.#method_name(stream).await {
                            Ok(x) => {
                                let x2: #method_output_as_in = x.into();
                                Some(x2.into_wasm())
                            },
                            Err(e) => {
                                // log error
                                log::error!("service:{}|method:{}|error:{}", self.service.descriptor(), #method_name_str, e);
                                None
                            }
                        }
                    }
                });
            },
            (false, true) => {
                // 1 -> many
                for field in &input_type.field {
                    //let param_name = quote::format_ident!("val{}", i.to_string());
                    let type_enum = ProtobufType::from_field(field, &service.name, false);
                    //let rs_type_name = type_enum.to_tokens();
                    let js_type_name = type_enum.to_wasm_tokens();
                    let rs_type_name = type_enum.to_tokens();
                    let field_name = quote::format_ident!(
                        "{}",
                        field
                            .name
                            .as_ref()
                            .expect("Protobuf message field needs a name")
                    );
                    input_params.push(quote::quote! {
                        #field_name: #js_type_name,
                    });
                    params_to_fields.push(quote::quote! {
                        #field_name: #rs_type_name::from_wasm(#field_name.into()),//: #field_name,
                    });
                }
                let params_to_fields_transformer = if input_type.field.len() == 1 {
                    let field_name = quote::format_ident!(
                        "{}",
                        input_type.field[0]
                            .name
                            .as_ref()
                            .expect("Protobuf message field needs a name")
                    );
                    quote::quote! {
                        let val = #method_input::from_wasm(#field_name.into());
                    }
                } else if input_type.field.is_empty() {
                    quote::quote! {
                        let val = #method_input {};
                    }
                } else {
                    quote::quote! {
                        let val = #method_input {
                            #(#params_to_fields)*
                        };
                    }
                };

                gen_methods.push(quote::quote! {
                    #[wasm_bindgen]
                    pub async fn #method_name(&self, #(#input_params)*, callback: js_sys::Function) {

                        #params_to_fields_transformer

                        match self.service.#method_name(val.into()).await {
                            Ok(x) => {
                                while let Some(next_result) = x.next().await {
                                    match next_result {
                                        Err(e) => {
                                            log::error!("service:{}|method:{}|error:{}", self.service.descriptor(), #method_name_str, e);
                                        },
                                        Ok(item) => {
                                            callback.call1(JsValue::undefined(), item.into_wasm_streamable());
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                // log error
                                log::error!("service:{}|method:{}|error:{}", self.service.descriptor(), #method_name_str, e);
                            }
                        }
                    }
                });
            },
            (true, true) => {
                // many -> many
                gen_methods.push(quote::quote! {
                    #[wasm_bindgen]
                    pub async fn #method_name(&self, generator: js_sys::Function, callback: js_sys::Function) -> Option<#method_output> {

                        // function into Rust futures Stream
                        let stream = Box::new(::usdpl_front::wasm::JsFunctionStream::<#method_input>::from_function(generator));

                        match self.service.#method_name(stream).await {
                            Ok(x) => {
                                while let Some(next_result) = x.next().await {
                                    match next_result {
                                        Err(e) => {
                                            log::error!("service:{}|method:{}|error:{}", self.service.descriptor(), #method_name_str, e);
                                        },
                                        Ok(item) => {
                                            callback.call1(JsValue::undefined(), item.into_wasm_streamable());
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                // log error
                                log::error!("service:{}|method:{}|error:{}", self.service.descriptor(), #method_name_str, e);
                                None
                            }
                        }
                    }
                });
            },
        }




    }
    quote::quote! {
        #(#gen_methods)*
    }
}

fn find_message_type<'a>(
    want_type: &str,
    want_package: &str,
    fds: &'a FileDescriptorSet,
) -> Option<&'a DescriptorProto> {
    for file in &fds.file {
        if let Some(pkg) = &file.package {
            if pkg == want_package {
                for message_type in &file.message_type {
                    if let Some(name) = &message_type.name {
                        if name == want_type {
                            return Some(message_type);
                        }
                    }
                }
            }
        }
    }
    None
}

fn find_enum_type<'a>(
    want_type: &str,
    want_package: &str,
    fds: &'a FileDescriptorSet,
) -> Option<&'a EnumDescriptorProto> {
    for file in &fds.file {
        for enum_type in &file.enum_type {
            if let Some(name) = &enum_type.name {
                if let Some(pkg) = &file.package {
                    if name == want_type && pkg == want_package {
                        return Some(enum_type);
                    }
                }
            }
        }
    }
    None
}

fn find_field<'a>(
    want_field: &str,
    descriptor: &'a DescriptorProto,
) -> Option<&'a FieldDescriptorProto> {
    for field in &descriptor.field {
        if let Some(name) = &field.name {
            if name == want_field {
                return Some(field);
            }
        }
    }
    None
}

fn is_known_map(field: &FieldDescriptorProto, known_maps: &HashSet<String>) -> bool {
    if let Some(type_name) = &field.type_name {
        let name = type_name.split('.').last().unwrap();
        known_maps.contains(name)
    } else {
        false
    }
}

fn generate_wasm_struct_interop(
    descriptor: &DescriptorProto,
    handled_enums: &mut HashSet<String>,
    handled_types: &mut HashSet<String>,
    known_maps: &mut HashSet<String>,
    seen_super_enums: &mut HashSet<String>,
    is_response_msg: bool,
    service: &str,
) -> proc_macro2::TokenStream {
    let msg_name = quote::format_ident!(
        "{}{}",
        service,
        descriptor
            .name
            .as_ref()
            .expect("Protobuf message needs a name")
    );
    let msg_name_wasm = quote::format_ident!(
        "{}{}Wasm",
        service,
        descriptor
            .name
            .as_ref()
            .expect("Protobuf message needs a name")
    );
    let super_msg_name = quote::format_ident!(
        "{}",
        descriptor
            .name
            .as_ref()
            .expect("Protobuf message needs a name")
    );
    let js_map_name = quote::format_ident!("{}", "js_map");
    let mut gen_fields = Vec::with_capacity(descriptor.field.len());
    let mut gen_into_fields = Vec::with_capacity(descriptor.field.len());
    let mut gen_from_fields = Vec::with_capacity(descriptor.field.len());

    let mut gen_nested_types = Vec::with_capacity(descriptor.nested_type.len());

    let mut gen_enums = Vec::with_capacity(descriptor.enum_type.len());

    if let Some(options) = &descriptor.options {
        //dbg!(options);
        if let Some(map_entry) = options.map_entry {
            // TODO deal with options when necessary
            if map_entry {
                //dbg!(descriptor);
                let name = descriptor
                    .name
                    .clone()
                    .expect("Protobuf message needs a name");
                known_maps.insert(name.clone());
                let key_field =
                    find_field("key", descriptor).expect("Protobuf map entry has no key field");
                let key_type = ProtobufType::from_field(&key_field, service, false);
                let value_field =
                    find_field("value", descriptor).expect("Protobuf map entry has no value field");
                let value_type = ProtobufType::from_field(&value_field, service, false);

                let map_type = ProtobufType::Map {
                    key: Box::new(key_type),
                    value: Box::new(value_type),
                };

                //dbg!("Generated map type", name);

                let map_tokens = map_type.to_tokens();
                let wasm_tokens = map_type.to_wasm_tokens();

                return quote::quote! {
                    pub type #msg_name = #map_tokens;
                    pub type #msg_name_wasm = #wasm_tokens;
                };
            }
        }
        // TODO Deal with other message options when necessary
    }

    //dbg!(&descriptor.options);

    for n_type in &descriptor.nested_type {
        let type_name = n_type
            .name
            .clone()
            .expect("Protobuf nested message needs a name");
        if !handled_types.contains(&type_name) {
            handled_types.insert(type_name);
            gen_nested_types.push(generate_wasm_struct_interop(
                n_type,
                handled_enums,
                handled_types,
                known_maps,
                seen_super_enums,
                is_response_msg,
                service,
            ));
        }
    }

    for e_type in &descriptor.enum_type {
        let type_name = e_type.name.clone().expect("Protobuf enum needs a name");
        if !handled_enums.contains(&type_name) {
            handled_enums.insert(type_name);
            gen_enums.push(generate_wasm_enum_interop(
                e_type,
                service,
                seen_super_enums,
            ));
        }
    }

    if descriptor.field.len() == 0 {
        quote::quote! {
            pub type #msg_name = ();
            pub type #msg_name_wasm = #msg_name;

            #(#gen_nested_types)*

            #(#gen_enums)*
        }
    } else if descriptor.field.len() == 1 {
        let field = &descriptor.field[0];
        //dbg!(descriptor, field);
        let field_name_str = field
            .name
            .as_ref()
            .expect("Protobuf message field needs a name");
        let field_name = quote::format_ident!(
            "{}",
            field_name_str
        );
        let type_enum = ProtobufType::from_field(field, service, is_known_map(field, known_maps));
        let type_name = type_enum.to_tokens();
        let wasm_type_name = type_enum.to_wasm_tokens();

        let into_wasm_streamable = quote::quote!{self.into_wasm_streamable()};
        let from_wasm_streamable = quote::quote!{#type_name::from_wasm_streamable(js)};

        quote::quote! {
            pub type #msg_name = #type_name;
            pub type #msg_name_wasm = #wasm_type_name;

            impl std::convert::Into<super::#super_msg_name> for #msg_name {
                #[inline]
                fn into(self) -> super::#super_msg_name {
                    super::#super_msg_name {
                        #field_name: self
                    }
                }
            }

            impl std::convert::From<super::#super_msg_name> for #msg_name {
                #[inline]
                #[allow(unused_variables)]
                fn from(other: super::#super_msg_name) -> Self {
                    other.#field_name
                }
            }

            impl ::usdpl_front::wasm::FromWasmStreamableType for #msg_name {
                fn from_wasm_streamable(js: JsValue) -> Result<Self, ::usdpl_front::wasm::WasmStreamableConversionError> {
                    #from_wasm_streamable
                }
            }

            impl ::usdpl_front::wasm::IntoWasmStreamableType for #msg_name {
                fn into_wasm_streamable(self) -> JsValue {
                    #into_wasm_streamable
                }
            }

            #(#gen_nested_types)*

            #(#gen_enums)*
        }
    } else {
        let mut gen_into_wasm_streamable_fields = Vec::with_capacity(descriptor.field.len());
        let mut gen_from_wasm_streamable_fields = Vec::with_capacity(descriptor.field.len());

        for field in &descriptor.field {
            let field_name_str = field
                .name
                .as_ref()
                .expect("Protobuf message field needs a name");
            let field_name = quote::format_ident!(
                "{}",
                field_name_str
            );
            let type_enum =
                ProtobufType::from_field(field, service, is_known_map(field, known_maps));
            let type_name = type_enum.to_tokens();

            let into_wasm_streamable = type_enum.to_into_wasm_streamable(field_name_str, &js_map_name);
            let from_wasm_streamable = type_enum.to_from_wasm_streamable(field_name_str, &js_map_name);
            //let wasm_type_name = type_enum.to_wasm_tokens();
            gen_fields.push(quote::quote! {
                pub #field_name: #type_name,
            });
            gen_into_fields.push(quote::quote! {
                #field_name: self.#field_name.into(),
            });

            gen_from_fields.push(quote::quote! {
                #field_name: <_>::from(other.#field_name),
            });

            gen_into_wasm_streamable_fields.push(into_wasm_streamable);
            gen_from_wasm_streamable_fields.push(from_wasm_streamable);
        }

        let wasm_attribute_maybe =
            if (descriptor.field.len() == 1 || !is_response_msg) && !descriptor.field.is_empty() {
                quote::quote! {}
            } else {
                quote::quote! {
                    #[wasm_bindgen]
                }
            };

        quote::quote! {
            #wasm_attribute_maybe
            pub struct #msg_name {
                #(#gen_fields)*
            }

            impl KnownWasmCompatible for #msg_name {}

            impl IntoWasmable<#msg_name> for #msg_name {
                fn into_wasm(self) -> Self {
                    self
                }
            }

            impl FromWasmable<#msg_name> for #msg_name {
                fn from_wasm(x: Self) -> Self {
                    x
                }
            }

            type #msg_name_wasm = #msg_name;

            impl std::convert::Into<super::#super_msg_name> for #msg_name {
                #[inline]
                fn into(self) -> super::#super_msg_name {
                    super::#super_msg_name {
                        #(#gen_into_fields)*
                    }
                }
            }

            impl std::convert::From<super::#super_msg_name> for #msg_name {
                #[inline]
                #[allow(unused_variables)]
                fn from(other: super::#super_msg_name) -> Self {
                    #msg_name {
                        #(#gen_from_fields)*
                    }
                }
            }

            impl ::usdpl_front::wasm::FromWasmStreamableType for #msg_name {
                fn from_wasm_streamable(js: JsValue) -> Result<Self, ::usdpl_front::wasm::WasmStreamableConversionError> {
                    let #js_map_name = js_sys::Map::from(js);
                    Ok(Self {
                        #(#gen_from_wasm_streamable_fields)*
                    })
                }
            }

            impl ::usdpl_front::wasm::IntoWasmStreamableType for #msg_name {
                fn into_wasm_streamable(self) -> JsValue {
                    let #js_map_name = js_sys::Map::new();
                    #(#gen_into_wasm_streamable_fields)*
                    #js_map_name.into()
                }
            }

            #(#gen_nested_types)*

            #(#gen_enums)*
        }
    }
}

#[derive(Debug)]
enum ProtobufType {
    Double,
    Float,
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Fixed32,
    Fixed64,
    Sfixed32,
    Sfixed64,
    Bool,
    String,
    Bytes,
    Repeated(Box<ProtobufType>),
    Map {
        key: Box<ProtobufType>,
        value: Box<ProtobufType>,
    },
    Custom(String),
}

impl ProtobufType {
    fn from_str(type_name: &str, service: &str) -> Self {
        match type_name {
            "double" => Self::Double,
            "float" => Self::Float,
            "int32" => Self::Int32,
            "int64" => Self::Int64,
            "uint32" => Self::Uint32,
            "uint64" => Self::Uint64,
            "sint32" => Self::Sint32,
            "sint64" => Self::Sint64,
            "fixed32" => Self::Fixed32,
            "fixed64" => Self::Fixed64,
            "sfixed32" => Self::Sfixed32,
            "sfixed64" => Self::Sfixed64,
            "bool" => Self::Bool,
            "string" => Self::String,
            "bytes" => Self::Bytes,
            t => Self::Custom(format!("{}{}", service, t.split('.').last().unwrap())),
        }
    }

    fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Double,
            //"float" => quote::quote!{f32},
            //"int32" => quote::quote!{i32},
            //"int64" => quote::quote!{i64},
            //"uint32" => quote::quote!{u32},
            4 => Self::Uint64,
            5 => Self::Int32,
            //"sint32" => quote::quote!{i32},
            //"sint64" => quote::quote!{i64},
            //"fixed32" => quote::quote!{u32},
            //"fixed64" => quote::quote!{u64},
            //"sfixed32" => quote::quote!{i32},
            //"sfixed64" => quote::quote!{i64},
            8 => Self::Bool,
            9 => Self::String,
            13 => Self::Uint32,
            //"bytes" => quote::quote!{Vec<u8>},
            t => Self::Custom(format!("UnknownType{}", t)),
        }
    }

    fn from_field(field: &FieldDescriptorProto, service: &str, is_map: bool) -> Self {
        let inner = if let Some(type_name) = &field.type_name {
            Self::from_str(type_name, service)
        } else {
            let number = field.r#type.unwrap();
            Self::from_id(number)
        };
        if let Some(label) = field.label {
            match label {
                3 if !is_map => Self::Repeated(Box::new(inner)), // is also the label for maps for some reason...
                _ => inner,
            }
        } else {
            inner
        }
    }

    fn to_tokens(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Double => quote::quote! {f64},
            Self::Float => quote::quote! {f32},
            Self::Int32 => quote::quote! {i32},
            Self::Int64 => quote::quote! {i64},
            Self::Uint32 => quote::quote! {u32},
            Self::Uint64 => quote::quote! {u64},
            Self::Sint32 => quote::quote! {i32},
            Self::Sint64 => quote::quote! {i64},
            Self::Fixed32 => quote::quote! {u32},
            Self::Fixed64 => quote::quote! {u64},
            Self::Sfixed32 => quote::quote! {i32},
            Self::Sfixed64 => quote::quote! {i64},
            Self::Bool => quote::quote! {bool},
            Self::String => quote::quote! {String},
            Self::Bytes => quote::quote! {Vec<u8>},
            Self::Repeated(t) => {
                let inner = t.to_tokens();
                quote::quote! {Vec::<#inner>}
            }
            Self::Map { key, value } => {
                let key = key.to_tokens();
                let value = value.to_tokens();
                quote::quote! {std::collections::HashMap<#key, #value>}
            }
            Self::Custom(t) => {
                let ident = quote::format_ident!("{}", t);
                quote::quote! {#ident}
            }
        }
    }

    fn to_wasm_tokens(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Double => quote::quote! {f64},
            Self::Float => quote::quote! {f32},
            Self::Int32 => quote::quote! {i32},
            Self::Int64 => quote::quote! {i64},
            Self::Uint32 => quote::quote! {u32},
            Self::Uint64 => quote::quote! {u64},
            Self::Sint32 => quote::quote! {i32},
            Self::Sint64 => quote::quote! {i64},
            Self::Fixed32 => quote::quote! {u32},
            Self::Fixed64 => quote::quote! {u64},
            Self::Sfixed32 => quote::quote! {i32},
            Self::Sfixed64 => quote::quote! {i64},
            Self::Bool => quote::quote! {bool},
            Self::String => quote::quote! {String},
            Self::Bytes => quote::quote! {Vec<u8>},
            Self::Repeated(_) => quote::quote! {js_sys::Array},
            Self::Map { .. } => quote::quote! {js_sys::Map},
            Self::Custom(t) => {
                let ident = quote::format_ident!("{}Wasm", t);
                quote::quote! {#ident}
            }
        }
    }

    fn to_into_wasm_streamable(&self, field_name: &str, js_map_name: &syn::Ident) -> proc_macro2::TokenStream {
        //let type_tokens = self.to_tokens();
        //let field_ident = quote::format_ident!("{}", field_name);
        quote::quote!{#js_map_name.set(#field_name.into(), self.field_ident);}
    }

    fn to_from_wasm_streamable(&self, field_name: &str, js_map_name: &syn::Ident) -> proc_macro2::TokenStream {
        let type_tokens = self.to_tokens();
        //let field_ident = quote::format_ident!("{}", field_name);
        quote::quote!{#field_name: #type_tokens::from_wasm_streamable(#js_map_name.get(#field_name.into()))?,}
    }
}

fn generate_wasm_enum_interop(
    descriptor: &EnumDescriptorProto,
    service: &str,
    seen_super_enums: &mut HashSet<String>,
) -> proc_macro2::TokenStream {
    let enum_name = quote::format_ident!(
        "{}{}",
        service,
        descriptor
            .name
            .as_ref()
            .expect("Protobuf enum needs a name")
    );
    let enum_name_wasm = quote::format_ident!(
        "{}{}Wasm",
        service,
        descriptor
            .name
            .as_ref()
            .expect("Protobuf enum needs a name")
    );
    let super_enum_name = quote::format_ident!(
        "{}",
        descriptor
            .name
            .as_ref()
            .expect("Protobuf enum needs a name")
    );
    let mut gen_values = Vec::with_capacity(descriptor.value.len());
    let mut gen_into_values = Vec::with_capacity(descriptor.value.len());
    let mut gen_from_values = Vec::with_capacity(descriptor.value.len());
    if let Some(_options) = &descriptor.options {
        // TODO deal with options when necessary
        todo!("Deal with enum options when necessary");
    }
    for value in &descriptor.value {
        let val_name = quote::format_ident!(
            "{}",
            value
                .name
                .as_ref()
                .expect("Protobuf enum value needs a name")
        );
        if let Some(_val_options) = &value.options {
            // TODO deal with options when necessary
            todo!("Deal with enum value options when necessary");
        } else {
            if let Some(number) = &value.number {
                gen_values.push(quote::quote! {
                    #val_name = #number,
                });
            } else {
                gen_values.push(quote::quote! {
                    #val_name,
                });
            }
            gen_into_values.push(quote::quote! {
                Self::#val_name => super::#super_enum_name::#val_name,
            });

            gen_from_values.push(quote::quote! {
                super::#super_enum_name::#val_name => Self::#val_name,
            });
        }
    }

    let impl_wasm_compat = if seen_super_enums.contains(descriptor.name.as_ref().unwrap()) {
        quote::quote! {}
    } else {
        seen_super_enums.insert(descriptor.name.clone().unwrap());
        quote::quote! {
            //impl KnownWasmCompatible for super::#super_enum_name {}
        }
    };

    quote::quote! {
        #[wasm_bindgen]
        #[repr(i32)]
        #[derive(Clone, Copy)]
        pub enum #enum_name {
            #(#gen_values)*
        }

        type #enum_name_wasm = #enum_name;

        impl std::convert::Into<super::#super_enum_name> for #enum_name {
            fn into(self) -> super::#super_enum_name {
                match self {
                    #(#gen_into_values)*
                }
            }
        }

        impl std::convert::From<super::#super_enum_name> for #enum_name {
            fn from(other: super::#super_enum_name) -> Self {
                match other {
                    #(#gen_from_values)*
                }
            }
        }

        #impl_wasm_compat

        impl FromWasmable<i32> for #enum_name {
            fn from_wasm(js: i32) -> Self {
                #enum_name::from(super::#super_enum_name::from_i32(js).unwrap())
            }
        }

        impl IntoWasmable<i32> for #enum_name {
            fn into_wasm(self) -> i32 {
                self as i32
            }
        }

        impl From<i32> for #enum_name {
            fn from(other: i32) -> Self {
                #enum_name::from(super::#super_enum_name::from_i32(other).unwrap())
            }
        }

        impl Into<i32> for #enum_name {
            fn into(self) -> i32 {
                self as i32
            }
        }
    }
}

fn generate_service_io_types(
    service: &Service,
    fds: &FileDescriptorSet,
) -> proc_macro2::TokenStream {
    let mut gen_types = Vec::with_capacity(service.methods.len() * 2);
    let mut gen_enums = Vec::new();
    let mut handled_enums = HashSet::new();
    let mut handled_types = HashSet::new();
    let mut known_maps = HashSet::new();
    let mut seen_super_enums = HashSet::new();
    for method in &service.methods {
        if let Some(input_message) = find_message_type(&method.input_type, &service.package, fds) {
            let msg_name = input_message
                .name
                .clone()
                .expect("Protobuf message name required");
            if !handled_types.contains(&msg_name) {
                handled_types.insert(msg_name);
                gen_types.push(generate_wasm_struct_interop(
                    input_message,
                    &mut handled_enums,
                    &mut handled_types,
                    &mut known_maps,
                    &mut seen_super_enums,
                    false,
                    &service.name,
                ));
            }
        } else if let Some(input_enum) = find_enum_type(&method.input_type, &service.package, fds) {
            let enum_name = input_enum
                .name
                .clone()
                .expect("Protobuf enum name required");
            if !handled_enums.contains(&enum_name) {
                handled_enums.insert(enum_name);
                gen_enums.push(generate_wasm_enum_interop(
                    input_enum,
                    &service.name,
                    &mut seen_super_enums,
                ));
            }
        } else {
            panic!(
                "Cannot find input proto type/message {}/{} for method {}",
                service.package, method.input_type, method.name
            );
        }

        if let Some(output_message) = find_message_type(&method.output_type, &service.package, fds)
        {
            let msg_name = output_message
                .name
                .clone()
                .expect("Protobuf message name required");
            if !handled_types.contains(&msg_name) {
                handled_types.insert(msg_name);
                gen_types.push(generate_wasm_struct_interop(
                    output_message,
                    &mut handled_enums,
                    &mut handled_types,
                    &mut known_maps,
                    &mut seen_super_enums,
                    true,
                    &service.name,
                ));
            }
        } else if let Some(output_enum) = find_enum_type(&method.output_type, &service.package, fds)
        {
            let enum_name = output_enum
                .name
                .clone()
                .expect("Protobuf enum name required");
            if !handled_enums.contains(&enum_name) {
                handled_enums.insert(enum_name);
                gen_enums.push(generate_wasm_enum_interop(
                    output_enum,
                    &service.name,
                    &mut seen_super_enums,
                ));
            }
        } else {
            panic!(
                "Cannot find output proto type/message {}/{} for method {}",
                service.package, method.output_type, method.name
            );
        }
    }

    // always generate all enums, since they aren't encountered (ever, afaik) when generating message structs
    for file in &fds.file {
        if let Some(pkg) = &file.package {
            if pkg == &service.package {
                for enum_type in &file.enum_type {
                    let enum_name = enum_type.name.clone().expect("Protobuf enum name required");
                    if !handled_enums.contains(&enum_name) {
                        handled_enums.insert(enum_name);
                        gen_enums.push(generate_wasm_enum_interop(
                            enum_type,
                            &service.name,
                            &mut seen_super_enums,
                        ));
                    }
                }
            }
        }
    }
    quote::quote! {
        #(#gen_types)*

        #(#gen_enums)*
    }
}

impl IServiceGenerator for WasmServiceGenerator {
    fn generate(&mut self, service: Service) -> proc_macro2::TokenStream {
        let lock = self.shared.lock().expect("Cannot lock shared state");
        let fds = lock
            .fds
            .as_ref()
            .expect("FileDescriptorSet required for WASM service generator");
        let service_struct_name = quote::format_ident!("{}Client", service.name);
        let service_js_name = quote::format_ident!("{}", service.name);
        let service_methods = generate_service_methods(&service, fds);
        let service_types = generate_service_io_types(&service, fds);
        let mod_name = quote::format_ident!("js_{}", service.name.to_lowercase());
        quote::quote! {
            mod #mod_name {
                #![allow(dead_code, unused_imports)]
                use usdpl_front::_helpers::wasm_bindgen::prelude::*;
                use usdpl_front::_helpers::wasm_bindgen;
                use usdpl_front::_helpers::wasm_bindgen_futures;
                use usdpl_front::_helpers::js_sys;
                use usdpl_front::_helpers::log;
                use usdpl_front::_helpers::futures;
                use usdpl_front::_helpers::futures::StreamExt;
                use usdpl_front::_helpers::nrpc::ClientService;
                use usdpl_front::wasm::{IntoWasmStreamableType, FromWasmStreamableType};

                use usdpl_front::wasm::*;

                use usdpl_front::WebSocketHandler;

                #service_types

                /// WASM/JS-compatible wrapper of the Rust nRPC service
                #[wasm_bindgen]
                pub struct #service_js_name {
                    //#[wasm_bindgen(skip)]
                    service: super::#service_struct_name<WebSocketHandler>,
                }

                #[wasm_bindgen]
                impl #service_js_name {
                    #[wasm_bindgen(constructor)]
                    pub fn new(port: u16) -> Self {
                        let implementation = super::#service_struct_name::new(
                            WebSocketHandler::new(port)
                        );
                        Self {
                            service: implementation,
                        }
                    }

                    #service_methods
                }
            }
        }
    }
}
