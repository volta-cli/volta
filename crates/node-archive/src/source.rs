//! Defines a helper macro `define_source_trait!`, for both the `tarball` and `zip`
//! modules to be able to define their own `Source` trait.

macro_rules! define_source_trait {
    { $name:ident : $($supertypes:tt)* } => {
        /// A data source for fetching a Node archive. In Unix operating systems, this
        /// is required to implement `Read`. In Windows, this trait extends both `Read`
        /// and `Seek` so as to be able to traverse the contents of zip archives.
        pub trait $name: $($supertypes)* {
            /// Produces the uncompressed size of the archive in bytes, when available.
            /// In Windows, this is never available and always produces `None`. In other
            /// platforms, this is always available and always produces a `Some` value.
            fn uncompressed_size(&self) -> Option<u64>;

            /// Produces the compressed size of the archive in bytes.
            fn compressed_size(&self) -> u64;
        }
    }
}
