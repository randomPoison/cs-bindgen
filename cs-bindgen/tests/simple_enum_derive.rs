//! Tests for verifying that `#[cs_bindgen]` generates the correct bindings for
//! simple enums (i.e. enums that don't carry extra data).
//!
//! These tests primarily verify that the generated `from_abi` and `into_abi` methods
//! agree on the discriminant value for each variant of an enum. This is especially
//! important for simple enums since the generated code needs to correctly handle
//! custom discriminant values, which may be arbitrary expressions (including
//! references to constants).
//!
//! In order to verify that the implementations are in sync we do a round-trip
//! with each variant, i.e. pass the result of `into_abi` back through
//! `from_abi` and then verify that the result matches the original.

use cs_bindgen::{abi::Abi, prelude::*};
use strum::{EnumIter, IntoEnumIterator};

#[test]
fn simple_enum_round_trip() {
    #[cs_bindgen]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
    enum Simple {
        Zero,
        One,
        Two,
    }

    for variant in Simple::iter() {
        let abi = variant.into_abi();
        let result = unsafe { Simple::from_abi(abi) };
        assert_eq!(variant, result);
    }
}

#[test]
fn simple_enum_explicit_discriminants() {
    const DISCRIMINANT: isize = 45;

    #[cs_bindgen]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
    pub enum ExplicitDiscrim {
        Foo,
        Bar,
        Baz = 5,
        Baa,
        Bab,
        Quux = 1 + 2 + 3 + 4,
        Cool,
        Wool,
        SomeDiscriminant = DISCRIMINANT,
        AnotherOne,
        YetAnotherOne,
        Negative = -5,
        NegativePlusOne,
        NegativePlusTwo,
    }

    for variant in ExplicitDiscrim::iter() {
        let abi = variant.into_abi();
        let result = unsafe { ExplicitDiscrim::from_abi(abi) };
        assert_eq!(variant, result);
    }
}

#[test]
fn simple_enum_explicit_first_discriminant() {
    #[cs_bindgen]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
    enum FirstDiscriminant {
        Zero = 123,
        One,
        Two,
    }

    for variant in FirstDiscriminant::iter() {
        let abi = variant.into_abi();
        let result = unsafe { FirstDiscriminant::from_abi(abi) };
        assert_eq!(variant, result);
    }
}
