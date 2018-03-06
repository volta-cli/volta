//! Provides so-called [KNOWNFOLDERID constants](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx)
//! of Windows, i.e., the GUIDs associated with standard folders registered with the system as
//! [known folders](https://msdn.microsoft.com/en-us/library/windows/desktop/bb776911.aspx).

use guid::GUID;

/// The [`FOLDERID_LocalAppData`](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_localappdata)
/// GUID (`{F1B32785-6FBA-4FCF-9D55-7B8E7F157091}`).
pub const LOCAL_APP_DATA: GUID = guid!{"F1B32785-6FBA-4FCF-9D55-7B8E7F157091"};

/// The [`FOLDERID_ProgramData`](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programdata)
/// GUID (`{62AB5D82-FDC1-4DC3-A9DD-070D1D495D97}`).
pub const PROGRAM_DATA: GUID = guid!{"62AB5D82-FDC1-4DC3-A9DD-070D1D495D97"};

/// The [`FOLDERID_ProgramFiles`](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programfiles)
/// GUID (`{905E63B6-C1BF-494E-B29C-65B732D3D21A}`).
pub const PROGRAM_FILES: GUID = guid!{"905E63B6-C1BF-494E-B29C-65B732D3D21A"};

/// The [`FOLDERID_ProgramFilesX64`](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programfilesx64)
/// GUID (`{6D809377-6AF0-444B-8957-A3773F02200E}`).
pub const PROGRAM_FILES_X64: GUID = guid!{"6D809377-6AF0-444B-8957-A3773F02200E"};

/// The [`FOLDERID_ProgramFilesX86`](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx#folderid_programfilesx86)
/// GUID (`{7C5A40EF-A0FB-4BFC-874A-C0F2E0B9FA8E}`).
pub const PROGRAM_FILES_X86: GUID = guid!{"7C5A40EF-A0FB-4BFC-874A-C0F2E0B9FA8E"};
