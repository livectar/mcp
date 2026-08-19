use serde::{Deserialize, Serialize};

use super::super::identifiers::limits::TextChunkSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellTextKind {
    Text,
    Formula,
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
