use semver::Version;

/// A toolchain manifest.
pub struct Image {
    /// The pinned version of Node, under the `toolchain.node` key.
    pub node: Version,
    /// The pinned version of Node as a string.
    pub node_str: String,
    /// The pinned version of Yarn, under the `toolchain.yarn` key.
    pub yarn: Option<Version>,
    /// The pinned version of Yarn as a string.
    pub yarn_str: Option<String>,
}
