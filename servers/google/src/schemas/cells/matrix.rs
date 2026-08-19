use serde::{de::Error as _, Deserialize, Serialize};

use super::values::CellValue;

pub const MUTATION_CELL_FORMAT_DESCRIPTION: &str =
    "Each mutation cell must be an object: {kind:text,value:string}, {kind:number,value:number}, {kind:boolean,value:boolean}, {kind:empty}, or {kind:formula,value:string}. Values and rows are two-dimensional matrices of these objects.";

pub const MAX_MUTATION_ROWS: usize = 1_000;
pub const MAX_MUTATION_COLUMNS: usize = 1_000;
pub const MAX_MUTATION_CELLS: usize = 10_000;
pub const MAX_MUTATION_CELL_TEXT_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct CellMatrix(Vec<Vec<CellValue>>);

impl CellMatrix {
    pub fn new(rows: Vec<Vec<CellValue>>) -> Result<Self, String> {
        validate_rows(&rows, true)?;
        Ok(Self(rows))
    }

    pub fn rows(&self) -> &[Vec<CellValue>] {
        &self.0
    }

    pub fn row_count(&self) -> usize {
        self.0.len()
    }

    pub fn column_count(&self) -> usize {
        self.0.first().map_or(0, Vec::len)
    }

    pub fn cell_count(&self) -> usize {
        self.0.iter().map(Vec::len).sum()
    }
}

impl<'de> Deserialize<'de> for CellMatrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let rows = Vec::<Vec<CellValue>>::deserialize(deserializer)
            .map_err(|_| D::Error::custom(MUTATION_CELL_FORMAT_DESCRIPTION))?;
        Self::new(rows).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct CellRows(Vec<Vec<CellValue>>);

impl CellRows {
    pub fn new(rows: Vec<Vec<CellValue>>) -> Result<Self, String> {
        validate_rows(&rows, false)?;
        Ok(Self(rows))
    }

    pub fn rows(&self) -> &[Vec<CellValue>] {
        &self.0
    }

    pub fn cell_count(&self) -> usize {
        self.0.iter().map(Vec::len).sum()
    }
}

impl<'de> Deserialize<'de> for CellRows {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let rows = Vec::<Vec<CellValue>>::deserialize(deserializer)
            .map_err(|_| D::Error::custom(MUTATION_CELL_FORMAT_DESCRIPTION))?;
        Self::new(rows).map_err(serde::de::Error::custom)
    }
}

fn validate_rows(rows: &[Vec<CellValue>], require_rectangular: bool) -> Result<(), String> {
    if rows.is_empty() || rows.len() > MAX_MUTATION_ROWS {
        return Err(format!(
            "mutation values must contain between 1 and {MAX_MUTATION_ROWS} rows"
        ));
    }
    let first_width = rows.first().map_or(0, Vec::len);
    if first_width == 0 || first_width > MAX_MUTATION_COLUMNS {
        return Err(format!(
            "mutation values must contain between 1 and {MAX_MUTATION_COLUMNS} columns"
        ));
    }
    let mut cells = 0usize;
    for row in rows {
        if row.is_empty() || row.len() > MAX_MUTATION_COLUMNS {
            return Err(format!(
                "mutation values must contain between 1 and {MAX_MUTATION_COLUMNS} columns"
            ));
        }
        if require_rectangular && row.len() != first_width {
            return Err("write_range values must form a rectangular matrix".to_string());
        }
        cells = cells.checked_add(row.len()).ok_or_else(|| {
            "mutation values cell count is outside the supported range".to_string()
        })?;
        if cells > MAX_MUTATION_CELLS {
            return Err(format!(
                "mutation values must contain at most {MAX_MUTATION_CELLS} cells"
            ));
        }
        for value in row {
            let text_bytes = match value {
                CellValue::Text(value) | CellValue::Formula(value) => value.len(),
                CellValue::Empty | CellValue::Boolean(_) => 0,
                CellValue::Number(value) if !value.is_finite() => {
                    return Err("mutation numbers must be finite".to_string());
                }
                CellValue::Number(_) => 0,
            };
            if let CellValue::Formula(value) = value {
                if !value.starts_with('=') {
                    return Err("formula mutation cells must start with '='".to_string());
                }
            }
            if text_bytes > MAX_MUTATION_CELL_TEXT_BYTES {
                return Err(format!(
                    "mutation cell text must be at most {MAX_MUTATION_CELL_TEXT_BYTES} bytes"
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CellMatrix, CellValue};

    #[test]
    fn mutation_cells_are_bounded_and_typed() {
        assert!(CellMatrix::new(vec![vec![CellValue::Formula("SUM(A1)".to_string())]]).is_err());
        assert!(CellMatrix::new(vec![vec![CellValue::Number(f64::NAN)]]).is_err());
        assert!(CellMatrix::new(vec![
            vec![CellValue::Text("a".to_string())],
            vec![CellValue::Text("b".to_string()), CellValue::Empty],
        ])
        .is_err());
    }

    #[test]
    fn invalid_mutation_cells_explain_the_public_format() {
        let error = serde_json::from_str::<CellMatrix>(r#"[["Alice"]]"#).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("mutation cell"));
        assert!(message.contains("kind:text"));
        assert!(!message.contains("CellValue"));
    }
}
