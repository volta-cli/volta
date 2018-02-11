use guid::GUID;

// https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_localappdata
// {F1B32785-6FBA-4FCF-9D55-7B8E7F157091}
pub const LOCAL_APP_DATA: GUID = guid!(0xF1B32785, 0x6FBA, 0x4FCF, 0x9D55, 0x7B8E7F157091);

// https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programdata
// {62AB5D82-FDC1-4DC3-A9DD-070D1D495D97}
pub const PROGRAM_DATA: GUID = guid!(0x62AB5D82, 0xFDC1, 0x4DC3, 0xA9DD, 0x070D1D495D97);

// https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programfiles
// {905E63B6-C1BF-494E-B29C-65B732D3D21A}
pub const PROGRAM_FILES: GUID = guid!(0x905E63B6, 0xC1BF, 0x494E, 0xB29C, 0x65B732D3D21A);

// https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programfilesx64
// {6D809377-6AF0-444B-8957-A3773F02200E}
pub const PROGRAM_FILES_X64: GUID = guid!(0x6D809377, 0x6AF0, 0x444B, 0x8957, 0xA3773F02200E);

// https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programfilesx86
// {7C5A40EF-A0FB-4BFC-874A-C0F2E0B9FA8E}
pub const PROGRAM_FILES_X86: GUID = guid!(0x7C5A40EF, 0xA0FB, 0x4BFC, 0x874A, 0xC0F2E0B9FA8E);
