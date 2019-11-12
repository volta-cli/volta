use volta_layout_macro::layout;

pub use super::v0::VoltaHome;

layout! {
    pub struct VoltaInstall {
        "volta-shim[.exe]": shim_executable;
        "volta[.exe]": main_executable;
    }
}
