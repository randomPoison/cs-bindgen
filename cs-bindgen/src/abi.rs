//! Traits for defining type conversions when communicating with C# code.
//!
//! In order to receive a value from C#, or return one to C#, the type must be
//! converted to a format that's compatible with C#. Specifically, it must be
//! converted to a type that's compatible with the C ABI.
//!
//! # Type Conversions
//!
//! > TODO: Describe the three unsafe primitive traits and the two conversions traits.
//!
//! # Safety
//!
//! > TODO: Describe what safety invariants have to be upheld when implementing the
//! > unsafe traits.
//!
//! [nomicon-interop]: https://doc.rust-lang.org/nomicon/ffi.html#interoperability-with-foreign-code

use core::mem::MaybeUninit;
use std::{convert::TryInto, mem, slice, str};

/// The ABI-compatible equivalent to [`String`].
///
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
pub type RawString = RawVec<u8>;

/// The ABI-compatible equivalent to [`&str`].
///
/// [`&str`]: https://doc.rust-lang.org/std/primitive.str.html
pub type RawStr = RawSlice<u8>;

/// A value that is ABI-compatible with C#.
///
/// Any type that implements this trait will automatically implement [`AbiArgument`]
/// and [`AbiReturn`] as well. You should almost always prefer to implement this
/// trait for custom types instead of implementing [`AbiArgument`] or [`AbiReturn`]
/// directly.
///
/// See [the module level documentation](./index.html) for more.
///
/// [`AbiArgument`]: trait.AbiArgument.html
/// [`AbiReturn`]: trait.AbiReturn.html
pub unsafe trait AbiPrimitive: Copy {}

/// A value that can be returned from a Rust function when called from C#.
pub trait Abi {
    type Abi: AbiPrimitive;

    fn into_abi(self) -> Self::Abi;
    unsafe fn from_abi(abi: Self::Abi) -> Self;
}

macro_rules! abi_primitives {
    ($($ty:ty,)*) => {
        $(
            unsafe impl AbiPrimitive for $ty {}

            impl Abi for $ty {
                type Abi = Self;

                fn into_abi(self) -> Self::Abi {
                    self
                }

                unsafe fn from_abi(abi: Self::Abi) -> Self {
                    abi
                }
            }
        )*
    };
}

// All numeric types are valid ABI primitives.
abi_primitives! {
    i8,
    i16,
    i32,
    i64,
    isize,
    u8,
    u16,
    u32,
    u64,
    usize,
    f32,
    f64,
}

impl Abi for () {
    type Abi = u8;

    fn into_abi(self) -> Self::Abi {
        0
    }

    unsafe fn from_abi(_: Self::Abi) -> Self {
        ()
    }
}

// Pointers to any ABI primitive are also valid ABI primitives.
unsafe impl<'a, T> AbiPrimitive for &'a T {}
unsafe impl<T> AbiPrimitive for *const T {}
unsafe impl<T> AbiPrimitive for *mut T {}

impl<T> Abi for Box<T> {
    type Abi = *mut T;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        Box::from_raw(abi)
    }

    fn into_abi(self) -> Self::Abi {
        Box::into_raw(self)
    }
}

impl Abi for char {
    type Abi = u32;

    fn into_abi(self) -> Self::Abi {
        self.into()
    }

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        abi.try_into().unwrap_or_default()
    }
}

impl Abi for bool {
    type Abi = u8;

    fn into_abi(self) -> Self::Abi {
        self.into()
    }

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        abi != 0
    }
}

impl<T> Abi for Vec<T>
where
    T: Abi,
{
    type Abi = RawVec<T>;

    fn into_abi(self) -> Self::Abi {
        self.into()
    }

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        abi.into_vec()
    }
}

impl Abi for String {
    type Abi = RawVec<u8>;

    fn into_abi(self) -> Self::Abi {
        self.into()
    }

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        abi.into_string()
    }
}

impl<'a> Abi for &'a str {
    type Abi = RawSlice<u8>;

    fn into_abi(self) -> Self::Abi {
        self.into()
    }

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        abi.as_str_unchecked()
    }
}

/// Raw representation of a [`Vec`] compatible with FFI.
///
/// When converting a `Vec<T>` into a `RawVec<T>`, no conversion is performed for
/// the elements of the vec. Instead, the generated C# code is expected to determine
/// if conversion is needed or not, and either memcopy the entire array or to
/// individually convert each element. The `[cs_bindgen]` proc macro generates a
/// conversion function for each type that the C# code uses to perform this
/// conversion.
///
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
#[repr(C)]
pub struct RawVec<T> {
    pub ptr: *mut T,
    pub len: usize,
    pub capacity: usize,
}

impl<T> RawVec<T> {
    pub unsafe fn into_vec(self) -> Vec<T> {
        Vec::from_raw_parts(self.ptr, self.len, self.capacity)
    }
}

impl<T> Clone for RawVec<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            capacity: self.capacity,
        }
    }
}
impl<T> Copy for RawVec<T> {}

impl RawVec<u8> {
    /// Reconstructs the original string from its raw parts.
    ///
    /// # Safety
    ///
    /// `into_string` must only be called once per string instance. Calling it more than
    /// once on the same string will result in undefined behavior.
    pub unsafe fn into_string(self) -> String {
        String::from_raw_parts(self.ptr, self.len, self.capacity)
    }
}

unsafe impl<T> AbiPrimitive for RawVec<T> {}

impl<T> From<Vec<T>> for RawVec<T> {
    fn from(mut from: Vec<T>) -> Self {
        let raw = Self {
            ptr: from.as_mut_ptr(),
            len: from.len(),
            capacity: from.capacity(),
        };

        // Ensure that the string isn't de-allocated, effectively transferring ownership of
        // its data to the `RawString`.
        mem::forget(from);

        raw
    }
}

impl From<String> for RawVec<u8> {
    fn from(mut from: String) -> Self {
        let raw = Self {
            ptr: from.as_mut_ptr(),
            len: from.len(),
            capacity: from.capacity(),
        };

        // Ensure that the string isn't de-allocated, effectively transferring ownership of
        // its data to the `RawString`.
        mem::forget(from);

        raw
    }
}

/// Raw representation of a `&[T]`.
///
/// When converting a `&[T]` into a `RawSlice<T>`, no conversion is performed for
/// the elements of the slice. Instead, the generated C# code is expected to
/// determine if conversion is needed or not, and either memcopy the entire array or
/// individually convert each element. The `[cs_bindgen]` proc macro generates a
/// conversion function for each type that the C# code uses to perform this
/// conversion.
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RawSlice<T> {
    pub ptr: *const T,
    pub len: usize,
}

impl<T> RawSlice<T> {
    pub unsafe fn as_slice<'a>(self) -> &'a [T] {
        slice::from_raw_parts(self.ptr, self.len)
    }
}

impl RawSlice<u8> {
    pub unsafe fn as_str<'a>(self) -> Result<&'a str, str::Utf8Error> {
        str::from_utf8(slice::from_raw_parts(self.ptr, self.len))
    }

    pub unsafe fn as_str_unchecked<'a>(self) -> &'a str {
        str::from_utf8_unchecked(slice::from_raw_parts(self.ptr, self.len))
    }
}

impl RawSlice<u16> {
    pub unsafe fn into_string(self) -> Result<String, std::string::FromUtf16Error> {
        let chars = slice::from_raw_parts(self.ptr, self.len as usize);

        // TODO: Is a lossy conversion the thing to do here?
        String::from_utf16(chars)
    }

    pub unsafe fn into_string_lossy(self) -> String {
        let chars = slice::from_raw_parts(self.ptr, self.len as usize);

        // TODO: Is a lossy conversion the thing to do here?
        String::from_utf16_lossy(chars)
    }
}

impl<T> Clone for RawSlice<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
        }
    }
}

impl<T> Copy for RawSlice<T> {}

unsafe impl<T> AbiPrimitive for RawSlice<T> {}

impl<'a, T> From<&'a [T]> for RawSlice<T>
where
    T: AbiPrimitive,
{
    fn from(from: &[T]) -> Self {
        Self {
            ptr: from.as_ptr(),
            len: from.len(),
        }
    }
}

impl<'a> From<&'a str> for RawSlice<u8> {
    fn from(from: &str) -> Self {
        Self {
            ptr: from.as_ptr(),
            len: from.len(),
        }
    }
}

macro_rules! array_abi {
    ( $len:expr; $($elem:ident),* ) => {
        unsafe impl<T: AbiPrimitive> AbiPrimitive for [T; $len] {}

        impl<T: Abi> Abi for [T; $len] {
            type Abi = [T::Abi; $len];

            fn into_abi(self) -> Self::Abi {
                let [
                    $( $elem, )*
                ] = self;

                [
                    $(
                        $crate::abi::Abi::into_abi($elem),
                    )*
                ]
            }

            unsafe fn from_abi(abi: Self::Abi) -> Self {
                let [
                    $( $elem, )*
                ] = abi;

                [
                    $(
                        $crate::abi::Abi::from_abi($elem),
                    )*
                ]
            }
        }
    };
}

array_abi!(1; a);
array_abi!(2; a, b);
array_abi!(3; a, b, c);
array_abi!(4; a, b, c, d);
array_abi!(5; a, b, c, d, e);
array_abi!(6; a, b, c, d, e, f);
array_abi!(7; a, b, c, d, e, f, g);
array_abi!(8; a, b, c, d, e, f, g, h);
array_abi!(9; a, b, c, d, e, f, g, h, i);
array_abi!(10; a, b, c, d, e, f, g, h, i, j);
array_abi!(11; a, b, c, d, e, f, g, h, i, j, k);
array_abi!(12; a, b, c, d, e, f, g, h, i, j, k, l);
array_abi!(13; a, b, c, d, e, f, g, h, i, j, k, l, m);
array_abi!(14; a, b, c, d, e, f, g, h, i, j, k, l, m, n);
array_abi!(15; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o);
array_abi!(16; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p);
array_abi!(17; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q);
array_abi!(18; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r);
array_abi!(19; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s);
array_abi!(20; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t);
array_abi!(21; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u);
array_abi!(22; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v);
array_abi!(23; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w);
array_abi!(24; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x);
array_abi!(25; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y);
array_abi!(26; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z);
array_abi!(27; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, aa);
array_abi!(28; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, aa, bb);
array_abi!(29; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, aa, bb, cc);
array_abi!(30; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, aa, bb, cc, dd);
array_abi!(31; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, aa, bb, cc, dd, ee);
array_abi!(32; a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, aa, bb, cc, dd, ee, ff);

/// Deconstructed representation of an enum, compatible with FFI.
///
/// The raw representation of an enum is an explicit discriminant value paired with
/// a union of all the fields. When converting back from the raw representation, use
/// the value of the discriminant to determine which union field is valid.
#[repr(C)]
#[derive(Debug, Copy)]
pub struct RawEnum<D, V> {
    pub discriminant: D,
    pub value: MaybeUninit<V>,
}

impl<D: Clone, V: Copy> Clone for RawEnum<D, V> {
    fn clone(&self) -> Self {
        Self {
            discriminant: self.discriminant.clone(),
            value: self.value.clone(),
        }
    }
}

impl<D, V> RawEnum<D, V> {
    pub const fn new(discriminant: D, value: V) -> Self {
        Self {
            discriminant,
            value: MaybeUninit::new(value),
        }
    }

    pub const fn unit(discriminant: D) -> Self {
        Self {
            discriminant,
            value: MaybeUninit::uninit(),
        }
    }
}

unsafe impl<D: AbiPrimitive, V: AbiPrimitive> AbiPrimitive for RawEnum<D, V> {}
