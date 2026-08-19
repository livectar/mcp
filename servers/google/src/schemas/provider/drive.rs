use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct DriveFilesResponse {
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    pub files: Vec<DriveFile>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "webViewLink")]
    pub web_view_link: Option<String>,
}
