use self::primitive::AbiPrimitive;

/// A value that can be sent to C#.
pub trait IntoAbi {
    type Abi: AbiPrimitive;

    fn into_abi(self) -> Self::Abi;
}

/// A value that can be received from C#.
pub trait FromAbi {
    type Abi: AbiPrimitive;

    fn from_abi(abi: Self::Abi) -> Self;
}

// NOTE: `AbiPrimitive` is declared in a private submodule so that it cannot be
// named outside this crate. This prevents external code from implementing the trait
// for new types, allowing us to enforce which types are considered ABI-compatible.
mod primitive {
    /// A value that is ABI-compatible with C#.
    pub unsafe trait AbiPrimitive {}
}

macro_rules! abi_primitives {
    ($($ty:ty,)*) => {
        $(
            unsafe impl AbiPrimitive for $ty {}

            impl IntoAbi for $ty {
                type Abi = Self;
                fn into_abi(self) -> Self::Abi { self }
            }

            impl FromAbi for $ty {
                type Abi = Self;
                fn from_abi(abi: Self::Abi) -> Self { abi }
            }
        )*
    };
}

abi_primitives! {
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
    f32,
    f64,
}
