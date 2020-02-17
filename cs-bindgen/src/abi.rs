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
/// # Safety
///
/// This type must be FFI-compatible with the C ABI.
pub unsafe trait AbiPrimitive {}

/// A value that can be returned from a Rust function when called from C#.
pub trait IntoAbi {
    type Abi: AbiPrimitive;

    fn into_abi(self) -> Self::Abi;
}

/// A value that can be accepted as an argument to a Rust function when called from C#.
pub trait FromAbi {
    type Abi: AbiPrimitive;

    unsafe fn from_abi(abi: Self::Abi) -> Self;
}

macro_rules! abi_primitives {
    ($($ty:ty,)*) => {
        $(
            unsafe impl AbiPrimitive for $ty {}

            impl IntoAbi for $ty {
                type Abi = Self;

                fn into_abi(self) -> Self::Abi {
                    self
                }
            }

            impl FromAbi for $ty {
                type Abi = Self;

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

// NOTE: Unit is also a valid primitive but it's only valid as a return value, not
// an argument. As such, we don't include it with `abi_primitives!` and instead
// manually implement just `IntoAbi` for it.
unsafe impl AbiPrimitive for () {}

impl IntoAbi for () {
    type Abi = Self;

    fn into_abi(self) -> Self::Abi {
        self
    }
}

// Pointers to any ABI primitive are also valid ABI primitives.
unsafe impl<T> AbiPrimitive for Box<T> {}
unsafe impl<'a, T> AbiPrimitive for &'a T {}
unsafe impl<'a, T> AbiPrimitive for &'a mut T {}
unsafe impl<T> AbiPrimitive for *const T {}
unsafe impl<T> AbiPrimitive for *mut T {}

impl IntoAbi for char {
    type Abi = u32;

    fn into_abi(self) -> Self::Abi {
        self.into()
    }
}

impl FromAbi for char {
    type Abi = u32;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        abi.try_into().unwrap_or_default()
    }
}

impl IntoAbi for bool {
    type Abi = u8;

    fn into_abi(self) -> Self::Abi {
        self.into()
    }
}

impl FromAbi for bool {
    type Abi = u8;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        abi != 0
    }
}

impl IntoAbi for String {
    type Abi = RawVec<u8>;

    fn into_abi(self) -> Self::Abi {
        self.into()
    }
}

impl FromAbi for String {
    type Abi = RawSlice<u16>;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        // TODO: Don't panic I guess?
        abi.into_string().expect("Failed to decode UTF-16")
    }
}

impl<'a> IntoAbi for &'a str {
    type Abi = RawSlice<u8>;

    fn into_abi(self) -> Self::Abi {
        self.into()
    }
}

/// Raw representation of a [`String`] compatible with FFI.
///
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

unsafe impl<T> AbiPrimitive for RawVec<T> where T: AbiPrimitive {}

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

/// Raw representation of a `string` passed from C#.
///
/// C# strings are encoded as utf-16, so they're effectively passed to rust as a
/// `u16` slice. This struct contains the raw pieces necessary to reconstruct the
/// slice, and provides a helper method `into_string` to copy the data into a
/// `String`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

unsafe impl<T> AbiPrimitive for RawSlice<T> where T: AbiPrimitive {}

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
