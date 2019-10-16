// This file is generated by rust-protobuf 2.8.1. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `coordinator.proto`

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

/// Generated files are compatible only with the same version
/// of protobuf runtime.
const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_8_1;

#[derive(PartialEq,Clone,Default)]
pub struct RendezvousRequest {
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a RendezvousRequest {
    fn default() -> &'a RendezvousRequest {
        <RendezvousRequest as ::protobuf::Message>::default_instance()
    }
}

impl RendezvousRequest {
    pub fn new() -> RendezvousRequest {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for RendezvousRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> RendezvousRequest {
        RendezvousRequest::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<RendezvousRequest>(
                    "RendezvousRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn default_instance() -> &'static RendezvousRequest {
        static mut instance: ::protobuf::lazy::Lazy<RendezvousRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const RendezvousRequest,
        };
        unsafe {
            instance.get(RendezvousRequest::new)
        }
    }
}

impl ::protobuf::Clear for RendezvousRequest {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RendezvousRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RendezvousRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct RendezvousReply {
    // message fields
    pub response: RendezvousResponse,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a RendezvousReply {
    fn default() -> &'a RendezvousReply {
        <RendezvousReply as ::protobuf::Message>::default_instance()
    }
}

impl RendezvousReply {
    pub fn new() -> RendezvousReply {
        ::std::default::Default::default()
    }

    // .xain.protobuf.coordinator.RendezvousResponse response = 1;


    pub fn get_response(&self) -> RendezvousResponse {
        self.response
    }
    pub fn clear_response(&mut self) {
        self.response = RendezvousResponse::ACCEPT;
    }

    // Param is passed by value, moved
    pub fn set_response(&mut self, v: RendezvousResponse) {
        self.response = v;
    }
}

impl ::protobuf::Message for RendezvousReply {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_proto3_enum_with_unknown_fields_into(wire_type, is, &mut self.response, 1, &mut self.unknown_fields)?
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.response != RendezvousResponse::ACCEPT {
            my_size += ::protobuf::rt::enum_size(1, self.response);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if self.response != RendezvousResponse::ACCEPT {
            os.write_enum(1, self.response.value())?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> RendezvousReply {
        RendezvousReply::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeEnum<RendezvousResponse>>(
                    "response",
                    |m: &RendezvousReply| { &m.response },
                    |m: &mut RendezvousReply| { &mut m.response },
                ));
                ::protobuf::reflect::MessageDescriptor::new::<RendezvousReply>(
                    "RendezvousReply",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn default_instance() -> &'static RendezvousReply {
        static mut instance: ::protobuf::lazy::Lazy<RendezvousReply> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const RendezvousReply,
        };
        unsafe {
            instance.get(RendezvousReply::new)
        }
    }
}

impl ::protobuf::Clear for RendezvousReply {
    fn clear(&mut self) {
        self.response = RendezvousResponse::ACCEPT;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RendezvousReply {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RendezvousReply {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct HeartbeatRequest {
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a HeartbeatRequest {
    fn default() -> &'a HeartbeatRequest {
        <HeartbeatRequest as ::protobuf::Message>::default_instance()
    }
}

impl HeartbeatRequest {
    pub fn new() -> HeartbeatRequest {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for HeartbeatRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> HeartbeatRequest {
        HeartbeatRequest::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<HeartbeatRequest>(
                    "HeartbeatRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn default_instance() -> &'static HeartbeatRequest {
        static mut instance: ::protobuf::lazy::Lazy<HeartbeatRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const HeartbeatRequest,
        };
        unsafe {
            instance.get(HeartbeatRequest::new)
        }
    }
}

impl ::protobuf::Clear for HeartbeatRequest {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for HeartbeatRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for HeartbeatRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct HeartbeatReply {
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a HeartbeatReply {
    fn default() -> &'a HeartbeatReply {
        <HeartbeatReply as ::protobuf::Message>::default_instance()
    }
}

impl HeartbeatReply {
    pub fn new() -> HeartbeatReply {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for HeartbeatReply {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> HeartbeatReply {
        HeartbeatReply::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<HeartbeatReply>(
                    "HeartbeatReply",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn default_instance() -> &'static HeartbeatReply {
        static mut instance: ::protobuf::lazy::Lazy<HeartbeatReply> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const HeartbeatReply,
        };
        unsafe {
            instance.get(HeartbeatReply::new)
        }
    }
}

impl ::protobuf::Clear for HeartbeatReply {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for HeartbeatReply {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for HeartbeatReply {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum RendezvousResponse {
    ACCEPT = 0,
    LATER = 1,
}

impl ::protobuf::ProtobufEnum for RendezvousResponse {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<RendezvousResponse> {
        match value {
            0 => ::std::option::Option::Some(RendezvousResponse::ACCEPT),
            1 => ::std::option::Option::Some(RendezvousResponse::LATER),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [RendezvousResponse] = &[
            RendezvousResponse::ACCEPT,
            RendezvousResponse::LATER,
        ];
        values
    }

    fn enum_descriptor_static() -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("RendezvousResponse", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for RendezvousResponse {
}

impl ::std::default::Default for RendezvousResponse {
    fn default() -> Self {
        RendezvousResponse::ACCEPT
    }
}

impl ::protobuf::reflect::ProtobufValue for RendezvousResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x11coordinator.proto\x12\x19xain.protobuf.coordinator\"\x13\n\x11Rend\
    ezvousRequest\"\\\n\x0fRendezvousReply\x12I\n\x08response\x18\x01\x20\
    \x01(\x0e2-.xain.protobuf.coordinator.RendezvousResponseR\x08response\"\
    \x12\n\x10HeartbeatRequest\"\x10\n\x0eHeartbeatReply*+\n\x12RendezvousRe\
    sponse\x12\n\n\x06ACCEPT\x10\0\x12\t\n\x05LATER\x10\x012\xde\x01\n\x0bCo\
    ordinator\x12h\n\nRendezvous\x12,.xain.protobuf.coordinator.RendezvousRe\
    quest\x1a*.xain.protobuf.coordinator.RendezvousReply\"\0\x12e\n\tHeartbe\
    at\x12+.xain.protobuf.coordinator.HeartbeatRequest\x1a).xain.protobuf.co\
    ordinator.HeartbeatReply\"\0J\xaf\x03\n\x06\x12\x04\0\0\x16\x19\n\x08\n\
    \x01\x0c\x12\x03\0\0\x12\n\x08\n\x01\x02\x12\x03\x02\x08!\n\n\n\x02\x06\
    \0\x12\x04\x04\0\x07\x01\n\n\n\x03\x06\0\x01\x12\x03\x04\x08\x13\n\x0b\n\
    \x04\x06\0\x02\0\x12\x03\x05\x02@\n\x0c\n\x05\x06\0\x02\0\x01\x12\x03\
    \x05\x06\x10\n\x0c\n\x05\x06\0\x02\0\x02\x12\x03\x05\x11\"\n\x0c\n\x05\
    \x06\0\x02\0\x03\x12\x03\x05-<\n\x0b\n\x04\x06\0\x02\x01\x12\x03\x06\x02\
    =\n\x0c\n\x05\x06\0\x02\x01\x01\x12\x03\x06\x06\x0f\n\x0c\n\x05\x06\0\
    \x02\x01\x02\x12\x03\x06\x10\x20\n\x0c\n\x05\x06\0\x02\x01\x03\x12\x03\
    \x06+9\n\n\n\x02\x05\0\x12\x04\t\0\x0c\x01\n\n\n\x03\x05\0\x01\x12\x03\t\
    \x05\x17\n\x0b\n\x04\x05\0\x02\0\x12\x03\n\x02\r\n\x0c\n\x05\x05\0\x02\0\
    \x01\x12\x03\n\x02\x08\n\x0c\n\x05\x05\0\x02\0\x02\x12\x03\n\x0b\x0c\n\
    \x0b\n\x04\x05\0\x02\x01\x12\x03\x0b\x02\x0c\n\x0c\n\x05\x05\0\x02\x01\
    \x01\x12\x03\x0b\x02\x07\n\x0c\n\x05\x05\0\x02\x01\x02\x12\x03\x0b\n\x0b\
    \n\t\n\x02\x04\0\x12\x03\x0e\0\x1c\n\n\n\x03\x04\0\x01\x12\x03\x0e\x08\
    \x19\n\n\n\x02\x04\x01\x12\x04\x10\0\x12\x01\n\n\n\x03\x04\x01\x01\x12\
    \x03\x10\x08\x17\n\x0b\n\x04\x04\x01\x02\0\x12\x03\x11\x02\"\n\r\n\x05\
    \x04\x01\x02\0\x04\x12\x04\x11\x02\x10\x19\n\x0c\n\x05\x04\x01\x02\0\x06\
    \x12\x03\x11\x02\x14\n\x0c\n\x05\x04\x01\x02\0\x01\x12\x03\x11\x15\x1d\n\
    \x0c\n\x05\x04\x01\x02\0\x03\x12\x03\x11\x20!\n\t\n\x02\x04\x02\x12\x03\
    \x14\0\x1b\n\n\n\x03\x04\x02\x01\x12\x03\x14\x08\x18\n\t\n\x02\x04\x03\
    \x12\x03\x16\0\x19\n\n\n\x03\x04\x03\x01\x12\x03\x16\x08\x16b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
