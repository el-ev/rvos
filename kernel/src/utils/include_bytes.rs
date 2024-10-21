
/// Align struct
#[repr(C)]
pub struct AlignedAs<Align, Bytes: ?Sized> {
    /// Align value
    pub _align: [Align; 0],
    /// Align bytes
    pub bytes: Bytes,
}

/// Include a file as a byte slice aligned as a specific type.
#[macro_export]
macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:literal) => {{
        use $crate::utils::include_bytes::AlignedAs;

        static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
            _align: [],
            bytes: *include_bytes!($path),
        };

        &ALIGNED.bytes
    }};
}
