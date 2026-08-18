use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoogleScope {
    #[serde(rename = "https://www.googleapis.com/auth/spreadsheets.readonly")]
    SheetsReadonly,
    #[serde(rename = "https://www.googleapis.com/auth/spreadsheets")]
    Sheets,
    #[serde(rename = "https://www.googleapis.com/auth/drive.readonly")]
    DriveReadonly,
}

impl GoogleScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SheetsReadonly => "https://www.googleapis.com/auth/spreadsheets.readonly",
            Self::Sheets => "https://www.googleapis.com/auth/spreadsheets",
            Self::DriveReadonly => "https://www.googleapis.com/auth/drive.readonly",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoogleSheetsScopeProfile;

impl GoogleSheetsScopeProfile {
    pub const fn read_only() -> [GoogleScope; 2] {
        [GoogleScope::SheetsReadonly, GoogleScope::DriveReadonly]
    }

    pub const fn mutations() -> [GoogleScope; 1] {
        [GoogleScope::Sheets]
    }
}
