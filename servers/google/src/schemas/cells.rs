use mcp_sdk::schemas::pagination::{OpaqueCursor, Paginated};
use serde::{Deserialize, Serialize};

use super::identifiers::{
    ids::SpreadsheetId,
    limits::{CellLimit, TextChunkSize},
    ranges::A1Range,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueRenderMode {
    Formatted,
    Unformatted,
    Formula,
}

impl Default for ValueRenderMode {
    fn default() -> Self {
        Self::Formatted
    }
}

impl ValueRenderMode {
    pub const fn provider_value(self) -> &'static str {
        match self {
            Self::Formatted => "FORMATTED_VALUE",
            Self::Unformatted => "UNFORMATTED_VALUE",
            Self::Formula => "FORMULA",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum CellValue {
    Empty,
    Text(String),
    Number(f64),
    Boolean(bool),
    Formula(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellTextKind {
    Text,
    Formula,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadRangeIdentity {
    pub spreadsheet_id: SpreadsheetId,
    pub tab: super::results::SheetIdentity,
    pub requested_range: A1Range,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadRangeResult {
    pub identity: ReadRangeIdentity,
    pub page_range: A1Range,
    pub values: Vec<Vec<CellValue>>,
    pub returned_cell_count: u32,
    pub next_cursor: Option<OpaqueCursor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadCellTextIdentity {
    pub spreadsheet_id: SpreadsheetId,
    pub tab: super::results::SheetIdentity,
    pub cell: A1Range,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadCellTextResult {
    pub identity: ReadCellTextIdentity,
    pub kind: CellTextKind,
    pub text: String,
    pub next_cursor: Option<OpaqueCursor>,
}

impl Paginated for ReadRangeResult {
    type Cursor = OpaqueCursor;

    fn next_cursor(&self) -> Option<&Self::Cursor> {
        self.next_cursor.as_ref()
    }
}

impl Paginated for ReadCellTextResult {
    type Cursor = OpaqueCursor;

    fn next_cursor(&self) -> Option<&Self::Cursor> {
        self.next_cursor.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RangeCursorPayload {
    pub spreadsheet_id: SpreadsheetId,
    pub range: A1Range,
    pub value_rendering: ValueRenderMode,
    pub max_cells: CellLimit,
    pub row_offset: u32,
    pub column_offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextChunkCursorPayload {
    pub spreadsheet_id: SpreadsheetId,
    pub cell: A1Range,
    pub value_rendering: ValueRenderMode,
    pub chunk_bytes: TextChunkSize,
    pub offset: u32,
}

pub(crate) fn split_text_chunk(
    value: &str,
    offset: u32,
    chunk_bytes: TextChunkSize,
) -> Result<(String, Option<u32>), String> {
    let offset =
        usize::try_from(offset).map_err(|_| "text cursor offset is invalid".to_string())?;
    if offset > value.len() || !value.is_char_boundary(offset) {
        return Err("text cursor offset is invalid".to_string());
    }
    if offset == value.len() {
        return Ok((String::new(), None));
    }
    let limit = usize::try_from(chunk_bytes.get()).unwrap_or(usize::MAX);
    let mut end = offset;
    for (relative, character) in value[offset..].char_indices() {
        let next = offset + relative + character.len_utf8();
        if next - offset > limit {
            break;
        }
        end = next;
    }
    if end == offset {
        return Err("chunk_bytes is too small for the next UTF-8 character".to_string());
    }
    let next_cursor = (end < value.len()).then_some(
        u32::try_from(end).map_err(|_| "cell text exceeds the cursor offset limit".to_string())?,
    );
    Ok((value[offset..end].to_string(), next_cursor))
}

#[cfg(test)]
mod tests {
    use super::{split_text_chunk, TextChunkSize};

    #[test]
    fn text_chunks_preserve_all_utf8_data() {
        let source = "🙂こんにちは".repeat(100);
        let chunk_size = TextChunkSize::new(256).unwrap();
        let mut offset = 0;
        let mut result = String::new();
        loop {
            let (chunk, next_offset) = split_text_chunk(&source, offset, chunk_size).unwrap();
            result.push_str(&chunk);
            let Some(next_offset) = next_offset else {
                break;
            };
            offset = next_offset;
        }

        assert_eq!(result, source);
    }
}
