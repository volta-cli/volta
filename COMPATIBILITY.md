# Compatibility

Volta currently tests against the following platforms, and will treat it as a breaking change to drop support for them:

- macOS
    - x86-64
    - Apple Silicon
- Linux x86-64
- Windows x86-64

We compile release artifacts compatible with the following, and likewise will treat it as a breaking change to drop support for them:

- macOS v11
- RHEL and CentOS v6
- Windows 10

In general, Volta should build and run against any other modern hardware and operating system supported by stable Rust, and we will make a best effort not to break them. However, we do *not* include them in our SemVer guarantees or test against them.
