//! Verify that handle types can contain types that aren't exported. Handle types
//! are meant to be opaque, so their contents shouldn't impact their ability to be
//! exported to C#.

use cs_bindgen::prelude::*;

#[cs_bindgen]
pub struct HandleType {
    pub non_exported_type: NonExportedType,
}

pub struct NonExportedType;
