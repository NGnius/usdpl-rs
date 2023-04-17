use std::collections::HashSet;

use prost_build::Service;
use prost_types::{FileDescriptorSet, DescriptorProto, EnumDescriptorProto, FieldDescriptorProto};
use nrpc_build::IServiceGenerator;

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

fn generate_service_methods(service: &Service, fds: &FileDescriptorSet) -> proc_macro2::TokenStream {
    let mut gen_methods = Vec::with_capacity(service.methods.len());
    for method in &service.methods {
        let method_name = quote::format_ident!("{}", method.name);
        let method_input = quote::format_ident!("{}{}", &service.name, method.input_type);
        let method_output = quote::format_ident!("{}{}", &service.name, method.output_type);

        let input_type = find_message_type(&method.input_type, &service.package, fds).expect("Protobuf message is used but not found");

        let mut input_params = Vec::with_capacity(input_type.field.len());
        let mut params_to_fields = Vec::with_capacity(input_type.field.len());
        for field in &input_type.field {
            //let param_name = quote::format_ident!("val{}", i.to_string());
            let type_name = translate_type(field, &service.name);
            let field_name = quote::format_ident!("{}", field.name.as_ref().expect("Protobuf message field needs a name"));
            input_params.push(quote::quote!{
                #field_name: #type_name,
            });
            params_to_fields.push(quote::quote!{
                #field_name,//: #field_name,
            });
        }
        let params_to_fields_transformer = if input_type.field.len() == 1 {
            let field_name = quote::format_ident!("{}", input_type.field[0].name.as_ref().expect("Protobuf message field needs a name"));
            quote::quote!{
                let val = #field_name;
            }
        } else if input_type.field.is_empty() {
            quote::quote!{
                let val = #method_input {};
            }
        } else {
            quote::quote!{
                let val = #method_input {
                    #(#params_to_fields)*
                };
            }
        };

        let special_fn_into_input = quote::format_ident!("{}_convert_into", method.input_type.split('.').last().unwrap().to_lowercase());

        let special_fn_from_output = quote::format_ident!("{}_convert_from", method.output_type.split('.').last().unwrap().to_lowercase());

        gen_methods.push(
            quote::quote!{
                #[wasm_bindgen]
                pub async fn #method_name(&mut self, #(#input_params)*) -> Option<#method_output> {

                    #params_to_fields_transformer

                    match self.service.#method_name(#special_fn_into_input(val)).await {
                        Ok(x) => Some(#special_fn_from_output(x)),
                        Err(_e) => {
                            // TODO log error
                            None
                        }
                    }
                }
            }
        );
    }
    quote::quote!{
        #(#gen_methods)*
    }
}

fn find_message_type<'a>(want_type: &str, want_package: &str, fds: &'a FileDescriptorSet) -> Option<&'a DescriptorProto> {
    for file in &fds.file {
        for message_type in &file.message_type {
            if let Some(name) = &message_type.name {
                if let Some(pkg) = &file.package {
                    if name == want_type && pkg == want_package {
                        return Some(message_type);
                    }
                }
            }
        }
    }
    None
}

fn find_enum_type<'a>(want_type: &str, want_package: &str, fds: &'a FileDescriptorSet) -> Option<&'a EnumDescriptorProto> {
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

fn find_field<'a>(want_field: &str, descriptor: &'a DescriptorProto) -> Option<&'a FieldDescriptorProto> {
    for field in &descriptor.field {
        if let Some(name) = &field.name {
            if name == want_field {
                return Some(field);
            }
        }
    }
    None
}

fn translate_type(field: &FieldDescriptorProto, service: &str) -> proc_macro2::TokenStream {
    if let Some(type_name) = &field.type_name {
        translate_type_name(type_name, service)
    } else {
        let number = field.r#type.unwrap();
        translate_type_known(number)
    }
}

fn generate_wasm_struct_interop(descriptor: &DescriptorProto, handled_enums: &mut HashSet<String>, handled_types: &mut HashSet<String>, is_response_msg: bool, service: &str) -> proc_macro2::TokenStream {
    let msg_name = quote::format_ident!("{}{}", service, descriptor.name.as_ref().expect("Protobuf message needs a name"));
    let super_msg_name = quote::format_ident!("{}", descriptor.name.as_ref().expect("Protobuf message needs a name"));
    let mut gen_fields = Vec::with_capacity(descriptor.field.len());
    let mut gen_into_fields = Vec::with_capacity(descriptor.field.len());
    let mut gen_from_fields = Vec::with_capacity(descriptor.field.len());

    let mut gen_nested_types = Vec::with_capacity(descriptor.nested_type.len());

    let mut gen_enums = Vec::with_capacity(descriptor.enum_type.len());

    if let Some(options) = &descriptor.options {
        if let Some(map_entry) = options.map_entry {
            // TODO deal with options when necessary
            if map_entry {
                let name = descriptor.name.clone().expect("Protobuf message needs a name");
                let special_fn_from = quote::format_ident!("{}_convert_from", name.split('.').last().unwrap().to_lowercase());
                let special_fn_into = quote::format_ident!("{}_convert_into", name.split('.').last().unwrap().to_lowercase());
                let key_field = find_field("key", descriptor).expect("Protobuf map entry has no key field");
                let key_type = translate_type(&key_field, service);
                let value_field = find_field("value", descriptor).expect("Protobuf map entry has no value field");
                let value_type = translate_type(&value_field, service);
                return quote::quote!{
                    pub type #msg_name = ::js_sys::Map;

                    #[inline]
                    #[allow(dead_code)]
                    fn #special_fn_from(other: ::std::collections::HashMap<#key_type, #value_type>) -> #msg_name {
                        let map = #msg_name::new();
                        for (key, val) in other.iter() {
                            map.set(&key.into(), &val.into());
                        }
                        map
                    }

                    #[inline]
                    #[allow(dead_code)]
                    fn #special_fn_into(this: #msg_name) -> ::std::collections::HashMap<#key_type, #value_type> {
                        let mut output = ::std::collections::HashMap::<#key_type, #value_type>::new();
                        this.for_each(&mut |key: ::wasm_bindgen::JsValue, val: ::wasm_bindgen::JsValue| {
                            if let Some(key) = key.as_string() {
                                if let Some(val) = val.as_string() {
                                    output.insert(key, val);
                                }
                            }
                        });
                        output
                    }
                }
            }
        } else {
            todo!("Deal with message options when necessary");
        }
    }

    for n_type in &descriptor.nested_type {
        let type_name = n_type.name.clone().expect("Protobuf nested message needs a name");
        if !handled_types.contains(&type_name) {
            handled_types.insert(type_name);
            gen_nested_types.push(generate_wasm_struct_interop(n_type, handled_enums, handled_types, is_response_msg, service));
        }
    }

    for e_type in &descriptor.enum_type {
        let type_name = e_type.name.clone().expect("Protobuf enum needs a name");
        if !handled_enums.contains(&type_name) {
            handled_enums.insert(type_name);
            gen_enums.push(generate_wasm_enum_interop(e_type, service));
        }
    }
    if descriptor.field.len() == 1 {
        let field = &descriptor.field[0];
        let field_name = quote::format_ident!("{}", field.name.as_ref().expect("Protobuf message field needs a name"));
        let type_name = translate_type(field, service);
        gen_fields.push(quote::quote!{
            pub #field_name: #type_name,
        });
        if let Some(name) = &field.type_name {
            let special_fn_from = quote::format_ident!("{}_convert_from", name.split('.').last().unwrap().to_lowercase());
            let special_fn_into = quote::format_ident!("{}_convert_into", name.split('.').last().unwrap().to_lowercase());
            gen_into_fields.push(
                quote::quote!{
                    #field_name: #special_fn_into(this)
                }
            );

            gen_from_fields.push(
                quote::quote!{
                    #special_fn_from(other.#field_name)
                }
            );
        } else {
            gen_into_fields.push(
                quote::quote!{
                    #field_name: this
                }
            );

            gen_from_fields.push(
                quote::quote!{
                    other.#field_name
                }
            );
        }

        let name = descriptor.name.clone().expect("Protobuf message needs a name");
        let special_fn_from = quote::format_ident!("{}_convert_from", name.split('.').last().unwrap().to_lowercase());
        let special_fn_into = quote::format_ident!("{}_convert_into", name.split('.').last().unwrap().to_lowercase());

        quote::quote!{
            pub type #msg_name = #type_name;

            #[inline]
            #[allow(dead_code)]
            fn #special_fn_from(other: super::#super_msg_name) -> #msg_name {
                #(#gen_from_fields)*
            }

            #[inline]
            #[allow(dead_code)]
            fn #special_fn_into(this: #msg_name) -> super::#super_msg_name {
                super::#super_msg_name {
                    #(#gen_into_fields)*
                }
            }

            #(#gen_nested_types)*

            #(#gen_enums)*
        }
    } else {
        for field in &descriptor.field {
            let field_name = quote::format_ident!("{}", field.name.as_ref().expect("Protobuf message field needs a name"));
            let type_name = translate_type(field, service);
            gen_fields.push(quote::quote!{
                pub #field_name: #type_name,
            });
            if let Some(name) = &field.type_name {
                let special_fn_from = quote::format_ident!("{}_convert_from", name.split('.').last().unwrap().to_lowercase());
                let special_fn_into = quote::format_ident!("{}_convert_into", name.split('.').last().unwrap().to_lowercase());
                gen_into_fields.push(
                    quote::quote!{
                        #field_name: #special_fn_into(self.#field_name),
                    }
                );

                gen_from_fields.push(
                    quote::quote!{
                        #field_name: #special_fn_from(other.#field_name),
                    }
                );
            } else {
                gen_into_fields.push(
                    quote::quote!{
                        #field_name: self.#field_name,
                    }
                );

                gen_from_fields.push(
                    quote::quote!{
                        #field_name: other.#field_name,
                    }
                );
            }
        }

        let name = descriptor.name.clone().expect("Protobuf message needs a name");
        let special_fn_from = quote::format_ident!("{}_convert_from", name.split('.').last().unwrap().to_lowercase());
        let special_fn_into = quote::format_ident!("{}_convert_into", name.split('.').last().unwrap().to_lowercase());

        let wasm_attribute_maybe = if descriptor.field.len() == 1 || !is_response_msg {
            quote::quote!{}
        } else {
            quote::quote!{
                #[wasm_bindgen]
            }
        };

        quote::quote!{
            #wasm_attribute_maybe
            pub struct #msg_name {
                #(#gen_fields)*
            }

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

            #[inline]
            #[allow(dead_code)]
            fn #special_fn_from(other: super::#super_msg_name) -> #msg_name {
                #msg_name::from(other)
            }

            #[inline]
            #[allow(dead_code)]
            fn #special_fn_into(this: #msg_name) -> super::#super_msg_name {
                this.into()
            }

            #(#gen_nested_types)*

            #(#gen_enums)*
        }
    }

}

fn translate_type_name(name: &str, service: &str) -> proc_macro2::TokenStream {
    match name {
        "double" => quote::quote!{f64},
        "float" => quote::quote!{f32},
        "int32" => quote::quote!{i32},
        "int64" => quote::quote!{i64},
        "uint32" => quote::quote!{u32},
        "uint64" => quote::quote!{u64},
        "sint32" => quote::quote!{i32},
        "sint64" => quote::quote!{i64},
        "fixed32" => quote::quote!{u32},
        "fixed64" => quote::quote!{u64},
        "sfixed32" => quote::quote!{i32},
        "sfixed64" => quote::quote!{i64},
        "bool" => quote::quote!{bool},
        "string" => quote::quote!{String},
        "bytes" => quote::quote!{Vec<u8>},
        t => {
            let ident = quote::format_ident!("{}{}", service, t.split('.').last().unwrap());
            quote::quote!{#ident}
        },
    }
}

fn translate_type_known(id: i32) -> proc_macro2::TokenStream {
    match id {
        //"double" => quote::quote!{f64},
        //"float" => quote::quote!{f32},
        //"int32" => quote::quote!{i32},
        //"int64" => quote::quote!{i64},
        //"uint32" => quote::quote!{u32},
        //"uint64" => quote::quote!{u64},
        //"sint32" => quote::quote!{i32},
        //"sint64" => quote::quote!{i64},
        //"fixed32" => quote::quote!{u32},
        //"fixed64" => quote::quote!{u64},
        //"sfixed32" => quote::quote!{i32},
        //"sfixed64" => quote::quote!{i64},
        //"bool" => quote::quote!{bool},
        9 => quote::quote!{String},
        //"bytes" => quote::quote!{Vec<u8>},
        t => {
            let ident = quote::format_ident!("UnknownType{}", t.to_string());
            quote::quote!{#ident}
        },
    }
}

fn generate_wasm_enum_interop(descriptor: &EnumDescriptorProto, service: &str) -> proc_macro2::TokenStream {
    let enum_name = quote::format_ident!("{}{}", service, descriptor.name.as_ref().expect("Protobuf enum needs a name"));
    let super_enum_name = quote::format_ident!("{}", descriptor.name.as_ref().expect("Protobuf enum needs a name"));
    let mut gen_values = Vec::with_capacity(descriptor.value.len());
    let mut gen_into_values = Vec::with_capacity(descriptor.value.len());
    let mut gen_from_values = Vec::with_capacity(descriptor.value.len());
    if let Some(_options) = &descriptor.options {
        // TODO deal with options when necessary
        todo!("Deal with enum options when necessary");
    }
    for value in &descriptor.value {
        let val_name = quote::format_ident!("{}", value.name.as_ref().expect("Protobuf enum value needs a name"));
        if let Some(_val_options) = &value.options {
            // TODO deal with options when necessary
            todo!("Deal with enum value options when necessary");
        } else {
            if let Some(number) = &value.number {
                gen_values.push(
                    quote::quote!{
                        #val_name = #number,
                    }
                );
            } else {
                gen_values.push(
                    quote::quote!{
                        #val_name,
                    }
                );
            }
            gen_into_values.push(
                quote::quote!{
                    Self::#val_name => super::#super_enum_name::#val_name,
                }
            );

            gen_from_values.push(
                quote::quote!{
                    super::#super_enum_name::#val_name => Self::#val_name,
                }
            );
        }
    }
    let name = descriptor.name.clone().expect("Protobuf message needs a name");
    let special_fn_from = quote::format_ident!("{}_convert_from", name.split('.').last().unwrap().to_lowercase());
    let special_fn_into = quote::format_ident!("{}_convert_into", name.split('.').last().unwrap().to_lowercase());

    quote::quote!{
        #[wasm_bindgen]
        #[repr(i32)]
        #[derive(Clone, Copy)]
        pub enum #enum_name {
            #(#gen_values)*
        }

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

        #[inline]
        #[allow(dead_code)]
        fn #special_fn_from(other: i32) -> #enum_name {
            #enum_name::from(super::#super_enum_name::from_i32(other).unwrap())
        }

        #[inline]
        #[allow(dead_code)]
        fn #special_fn_into(this: #enum_name) -> i32 {
            this as i32
        }
    }
}

fn generate_service_io_types(service: &Service, fds: &FileDescriptorSet) -> proc_macro2::TokenStream {
    let mut gen_types = Vec::with_capacity(service.methods.len() * 2);
    let mut gen_enums = Vec::new();
    let mut handled_enums = HashSet::new();
    let mut handled_types = HashSet::new();
    for method in &service.methods {
        if let Some(input_message) = find_message_type(&method.input_type, &service.package, fds) {
            let msg_name = input_message.name.clone().expect("Protobuf message name required");
            if !handled_types.contains(&msg_name) {
                handled_types.insert(msg_name);
                gen_types.push(generate_wasm_struct_interop(input_message, &mut handled_enums, &mut handled_types, false, &service.name));
            }
        } else if let Some(input_enum) = find_enum_type(&method.input_type, &service.package, fds) {
            let enum_name = input_enum.name.clone().expect("Protobuf enum name required");
            if !handled_enums.contains(&enum_name) {
                handled_enums.insert(enum_name);
                gen_types.push(generate_wasm_enum_interop(input_enum, &service.name));
            }
        } else {
            panic!("Cannot find proto type {}/{}", service.package, method.input_type);
        }

        if let Some(output_message) = find_message_type(&method.output_type, &service.package, fds) {
            let msg_name = output_message.name.clone().expect("Protobuf message name required");
            if !handled_types.contains(&msg_name) {
                handled_types.insert(msg_name);
                gen_types.push(generate_wasm_struct_interop(output_message, &mut handled_enums, &mut handled_types, true, &service.name));
            }
        } else if let Some(output_enum) = find_enum_type(&method.output_type, &service.package, fds) {
            let enum_name = output_enum.name.clone().expect("Protobuf enum name required");
            if !handled_enums.contains(&enum_name) {
                handled_enums.insert(enum_name);
                gen_types.push(generate_wasm_enum_interop(output_enum, &service.name));
            }
        } else {
            panic!("Cannot find proto type {}/{}", service.package, method.input_type);
        }
    }

    // always generate all enums, since they aren't encountered (ever, afaik) when generating message structs
    for file in &fds.file {
        for enum_type in &file.enum_type {
            let enum_name = enum_type.name.clone().expect("Protobuf enum name required");
            if !handled_enums.contains(&enum_name) {
                handled_enums.insert(enum_name);
                gen_enums.push(generate_wasm_enum_interop(enum_type, &service.name));
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
        let lock = self.shared.lock()
            .expect("Cannot lock shared state");
        let fds = lock.fds
            .as_ref()
            .expect("FileDescriptorSet required for WASM service generator");
        let service_struct_name = quote::format_ident!("{}Client", service.name);
        let service_js_name = quote::format_ident!("{}", service.name);
        let service_methods = generate_service_methods(&service, fds);
        let service_types = generate_service_io_types(&service, fds);
        let mod_name = quote::format_ident!("js_{}", service.name.to_lowercase());
        quote::quote!{
            mod #mod_name {
                use wasm_bindgen::prelude::*;

                use crate::client_handler::WebSocketHandler;

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
