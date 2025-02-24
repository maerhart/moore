// Copyright (c) 2016-2021 Fabian Schuiki

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub unsafe fn mlirStringRefCreateFromStr(s: impl AsRef<str>) -> MlirStringRef {
    let s = s.as_ref().as_bytes();
    MlirStringRef {
        data: s.as_ptr() as *const _,
        length: s.len() as size_t,
    }
}

pub unsafe fn mlirStringRefToStr<'a, R>(s: MlirStringRef, f: impl Fn(&'a str) -> R + 'a) -> R {
    f(std::str::from_utf8(std::slice::from_raw_parts(
        s.data as *const _,
        s.length as usize,
    ))
    .expect("utf8 string"))
}

pub unsafe fn mlirIdentifierGetFromStr(cx: MlirContext, s: impl AsRef<str>) -> MlirIdentifier {
    mlirIdentifierGet(cx, mlirStringRefCreateFromStr(s))
}

pub unsafe fn mlirIdentifierToStr<'a, R>(i: MlirIdentifier, f: impl Fn(&'a str) -> R + 'a) -> R {
    mlirStringRefToStr(mlirIdentifierStr(i), f)
}

impl Eq for MlirType {}

impl PartialEq for MlirType {
    fn eq(&self, other: &Self) -> bool {
        match (self.ptr.is_null(), other.ptr.is_null()) {
            (true, true) => true,
            (false, false) => unsafe { mlirTypeEqual(*self, *other) },
            _ => false,
        }
    }
}

impl Eq for MlirBlock {}

impl PartialEq for MlirBlock {
    fn eq(&self, other: &Self) -> bool {
        match (self.ptr.is_null(), other.ptr.is_null()) {
            (true, true) => true,
            (false, false) => unsafe { mlirBlockEqual(*self, *other) },
            _ => false,
        }
    }
}
