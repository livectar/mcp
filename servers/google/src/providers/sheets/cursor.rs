use crate::{
    errors::GoogleProviderError,
    schemas::{
        cells::cursors::{RangeCursorPayload, TextChunkCursorPayload},
        identifiers::limits::{CellLimit, TextChunkSize},
        requests::sheets_read::{ReadCellTextRequest, ReadRangeRequest},
    },
};

use super::validation::invalid_request;

pub(super) fn decode_range_cursor(
    request: &ReadRangeRequest,
    max_cells: CellLimit,
) -> Result<(u32, u32), GoogleProviderError> {
    let Some(cursor) = request.continuation_cursor.as_ref() else {
        return Ok((0, 0));
    };
    let payload = cursor
        .decode::<RangeCursorPayload>()
        .map_err(|error| invalid_request(error.to_string()))?;
    if payload.spreadsheet_id != request.spreadsheet_id
        || payload.range != request.range
        || payload.value_rendering != request.value_rendering
        || payload.max_cells != max_cells
    {
        return Err(invalid_request(
            "range continuation cursor does not match the requested read",
        ));
    }
    Ok((payload.row_offset, payload.column_offset))
}

pub(super) fn decode_text_cursor(
    request: &ReadCellTextRequest,
    chunk_bytes: TextChunkSize,
) -> Result<u32, GoogleProviderError> {
    let Some(cursor) = request.continuation_cursor.as_ref() else {
        return Ok(0);
    };
    let payload = cursor
        .decode::<TextChunkCursorPayload>()
        .map_err(|error| invalid_request(error.to_string()))?;
    if payload.spreadsheet_id != request.spreadsheet_id
        || payload.cell != request.cell
        || payload.value_rendering != request.value_rendering
        || payload.chunk_bytes != chunk_bytes
    {
        return Err(invalid_request(
            "text continuation cursor does not match the requested read",
        ));
    }
    Ok(payload.offset)
}
