//! Implementations of `Encode` for primitives and types provided by the standard libary.

use crate::encode::{Encode, Encoder};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};

/// Generates the `Encode` impl for primitives and collection types.
///
/// Forwards any specified generic parameters directly to the specified encode
/// function and automatically adds the `Encode` bound for generic parameters. For
/// example, invoking with `Collection<A, B> => encode_map` will expand to:
///
/// ```
/// impl<A, B> Encode for Collection<A, B> where A: Encode, B: Encode {
///     fn encode<E: Encoder>(encoder: E) -> Result<> {
///         encoder.encode_map::<A, B>()
///     }
/// }
/// ```
macro_rules! impl_encode {
    ( $( $ty:ident $( < $( $generic:ident ),* > )? => $encode:ident, )* ) => {
        $(
            impl $( < $( $generic ),* > )? Encode for $ty $( < $( $generic ),* > where $( $generic: Encode ),* )? {
                fn encode<E: Encoder>(encoder: E) -> Result<E::Ok, E::Error> {
                    encoder.$encode $( ::<$( $generic, )* >)?()
                }
            }
        )*
    }
}

impl_encode! {
    i8 => encode_i8,
    i16 => encode_i16,
    i32 => encode_i32,
    i64 => encode_i64,
    i128 => encode_i128,
    u8 => encode_u8,
    u16 => encode_u16,
    u32 => encode_u32,
    u64 => encode_u64,
    u128 => encode_u128,
    bool => encode_bool,
    char => encode_char,
    String => encode_string,
    Option<T> => encode_option,
    Vec<T> => encode_seq,
    VecDeque<T> => encode_seq,
    HashMap<K, V> => encode_map,
    BTreeMap<K, V> => encode_map,
    HashSet<T> => encode_seq,
    BTreeSet<T> => encode_seq,
    BinaryHeap<T> => encode_seq,
    LinkedList<T> => encode_seq,
}

impl Encode for () {
    fn encode<E: Encoder>(encoder: E) -> Result<E::Ok, E::Error> {
        encoder.encode_unit()
    }
}

// impl<'a> Encode for &'a str {
//     fn encode<E: Encoder>(encoder: E) -> Result<E::Ok, E::Error> {
//         encoder.encode_str()
//     }
// }
