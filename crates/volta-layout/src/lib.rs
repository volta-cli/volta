use volta_layout_macro::layout;

layout! {
    pub struct VoltaInstall {
        "volta[.exe]": volta_file;
        "shim[.exe]": shim_executable;
    }
}
