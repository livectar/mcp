use crate::{
    errors::GoogleProviderError,
    schemas::identifiers::{limits::CellLimit, ranges::ResolvedA1RangeBounds},
};

use super::validation::invalid_request;

#[derive(Debug, Clone, Copy)]
pub(super) struct RangePage {
    pub(super) bounds: ResolvedA1RangeBounds,
    pub(super) row_offset: u32,
    pub(super) column_offset: u32,
    pub(super) start_row: u32,
    pub(super) start_column: u32,
    pub(super) row_count: u32,
    pub(super) column_count: u32,
}

impl RangePage {
    pub(super) fn new(
        bounds: ResolvedA1RangeBounds,
        row_offset: u32,
        column_offset: u32,
        max_cells: CellLimit,
    ) -> Result<Self, GoogleProviderError> {
        if row_offset >= bounds.row_count() || column_offset >= bounds.column_count() {
            return Err(invalid_request("range continuation cursor is exhausted"));
        }
        let remaining_rows = bounds.row_count() - row_offset;
        let remaining_columns = bounds.column_count() - column_offset;
        let column_count = remaining_columns.min(max_cells.get());
        let row_count = remaining_rows.min((max_cells.get() / column_count).max(1));
        Ok(Self {
            bounds,
            row_offset,
            column_offset,
            start_row: bounds.start_row + row_offset,
            start_column: bounds.start_column + column_offset,
            row_count,
            column_count,
        })
    }

    pub(super) fn next_offsets(self) -> Option<(u32, u32)> {
        if self.column_offset + self.column_count < self.bounds.column_count() {
            Some((self.row_offset, self.column_offset + self.column_count))
        } else if self.row_offset + self.row_count < self.bounds.row_count() {
            Some((self.row_offset + self.row_count, 0))
        } else {
            None
        }
    }
}
