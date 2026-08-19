use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

const MAX_RANGE_BYTES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct A1Range(String);

impl A1Range {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        validate_a1_range(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn sheet_name(&self) -> Option<String> {
        let (sheet, _) = self.0.split_once('!')?;
        Some(unquote_sheet_name(sheet))
    }

    pub fn bounds(&self) -> Result<A1RangeBounds, String> {
        let reference = self
            .0
            .split_once('!')
            .map(|(_, reference)| reference)
            .unwrap_or(self.as_str());
        let mut parts = reference.split(':');
        let start = parse_a1_endpoint(parts.next().unwrap_or_default())?;
        let end = parts
            .next()
            .map(parse_a1_endpoint)
            .transpose()?
            .unwrap_or(start);
        Ok(A1RangeBounds {
            start_column: start.column.or(end.column),
            end_column: end.column.or(start.column),
            start_row: start.row.or(end.row),
            end_row: end.row.or(start.row),
        })
    }

    pub fn is_single_cell(&self) -> bool {
        let Ok(bounds) = self.bounds() else {
            return false;
        };
        matches!(
            (
                bounds.start_column,
                bounds.end_column,
                bounds.start_row,
                bounds.end_row,
            ),
            (Some(start_column), Some(end_column), Some(start_row), Some(end_row))
                if start_column == end_column && start_row == end_row
        )
    }

    pub fn is_fully_bounded(&self) -> bool {
        let reference = self
            .0
            .split_once('!')
            .map(|(_, reference)| reference)
            .unwrap_or(self.as_str());
        let mut parts = reference.split(':');
        let Ok(start) = parse_a1_endpoint(parts.next().unwrap_or_default()) else {
            return false;
        };
        let Ok(end) = parts.next().map(parse_a1_endpoint).transpose() else {
            return false;
        };
        let end = end.unwrap_or(start);
        start.column.is_some() && start.row.is_some() && end.column.is_some() && end.row.is_some()
    }

    pub fn subrange(
        &self,
        start_row: u32,
        start_column: u32,
        row_count: u32,
        column_count: u32,
    ) -> Result<Self, String> {
        if start_row == 0 || start_column == 0 || row_count == 0 || column_count == 0 {
            return Err("A1 subrange coordinates and dimensions must be positive".to_string());
        }
        let end_row = start_row
            .checked_add(row_count - 1)
            .ok_or_else(|| "A1 subrange row exceeds the supported range".to_string())?;
        let end_column = start_column
            .checked_add(column_count - 1)
            .ok_or_else(|| "A1 subrange column exceeds the supported range".to_string())?;
        let sheet_prefix = self
            .as_str()
            .split_once('!')
            .map(|(sheet, _)| format!("{sheet}!"))
            .unwrap_or_default();
        Self::new(format!(
            "{sheet_prefix}{}{}:{}{}",
            column_label(start_column),
            start_row,
            column_label(end_column),
            end_row
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct A1RangeBounds {
    pub start_column: Option<u32>,
    pub end_column: Option<u32>,
    pub start_row: Option<u32>,
    pub end_row: Option<u32>,
}

impl A1RangeBounds {
    pub fn resolve(
        self,
        default_rows: Option<u32>,
        default_columns: Option<u32>,
    ) -> Result<ResolvedA1RangeBounds, String> {
        let (start_row, end_row) = resolve_axis(self.start_row, self.end_row, default_rows, "row")?;
        let (start_column, end_column) = resolve_axis(
            self.start_column,
            self.end_column,
            default_columns,
            "column",
        )?;
        Ok(ResolvedA1RangeBounds {
            start_row,
            end_row,
            start_column,
            end_column,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedA1RangeBounds {
    pub start_column: u32,
    pub end_column: u32,
    pub start_row: u32,
    pub end_row: u32,
}

impl ResolvedA1RangeBounds {
    pub fn row_count(self) -> u32 {
        self.end_row - self.start_row + 1
    }

    pub fn column_count(self) -> u32 {
        self.end_column - self.start_column + 1
    }
}

impl<'de> Deserialize<'de> for A1Range {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

fn validate_a1_range(value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > MAX_RANGE_BYTES || value.chars().any(char::is_control) {
        return Err("range must be non-empty, bounded, and control-character-free".to_string());
    }
    let (sheet, reference) = value
        .split_once('!')
        .map_or((None, value), |(sheet, reference)| (Some(sheet), reference));
    if let Some(sheet) = sheet {
        if sheet.is_empty() || !valid_sheet_name(sheet) {
            return Err("range contains an invalid sheet name".to_string());
        }
    }
    if reference.is_empty()
        || reference.contains('!')
        || reference.split(':').count() > 2
        || reference.split(':').any(|part| !valid_a1_reference(part))
    {
        return Err("range must use a valid A1 reference such as Sheet1!A1:C10".to_string());
    }
    Ok(())
}

fn valid_sheet_name(value: &str) -> bool {
    if value.starts_with('\'') {
        value.len() >= 2
            && value.ends_with('\'')
            && value[1..value.len() - 1]
                .split("''")
                .all(|part| !part.is_empty() && !part.chars().any(char::is_control))
    } else {
        !value.chars().any(|character| {
            character.is_whitespace() || matches!(character, '[' | ']' | ':' | '*')
        })
    }
}

fn unquote_sheet_name(value: &str) -> String {
    if value.starts_with('\'') && value.ends_with('\'') {
        value[1..value.len() - 1].replace("''", "'")
    } else {
        value.to_string()
    }
}

fn valid_a1_reference(value: &str) -> bool {
    if value.starts_with('$') {
        return valid_a1_reference(&value[1..]);
    }
    let mut letters = 0;
    let mut digits = 0;
    for character in value.chars() {
        if character.is_ascii_alphabetic() && digits == 0 {
            letters += 1;
        } else if character.is_ascii_digit() {
            digits += 1;
        } else {
            return false;
        }
    }
    (letters > 0 && digits > 0) || (letters > 0 && digits == 0) || (letters == 0 && digits > 0)
}

#[derive(Debug, Clone, Copy)]
struct A1Endpoint {
    column: Option<u32>,
    row: Option<u32>,
}

fn parse_a1_endpoint(value: &str) -> Result<A1Endpoint, String> {
    let value = value.strip_prefix('$').unwrap_or(value);
    if value.is_empty() {
        return Err("A1 reference endpoint must not be empty".to_string());
    }
    let split_at = value
        .char_indices()
        .find(|(_, character)| character.is_ascii_digit())
        .map(|(index, _)| index)
        .unwrap_or(value.len());
    let (letters, digits) = value.split_at(split_at);
    if letters.is_empty() && digits.is_empty()
        || letters
            .chars()
            .any(|character| !character.is_ascii_alphabetic())
        || digits.chars().any(|character| !character.is_ascii_digit())
    {
        return Err("A1 reference endpoint is invalid".to_string());
    }
    let column = if letters.is_empty() {
        None
    } else {
        Some(column_number(&letters.to_ascii_uppercase())?)
    };
    let row = if digits.is_empty() {
        None
    } else {
        let row = digits
            .parse::<u32>()
            .map_err(|_| "A1 row is outside the supported range".to_string())?;
        if row == 0 {
            return Err("A1 row numbers must be positive".to_string());
        }
        Some(row)
    };
    Ok(A1Endpoint { column, row })
}

fn column_number(value: &str) -> Result<u32, String> {
    let mut number = 0_u32;
    for character in value.bytes() {
        number = number
            .checked_mul(26)
            .and_then(|value| value.checked_add(u32::from(character - b'A' + 1)))
            .ok_or_else(|| "A1 column is outside the supported range".to_string())?;
    }
    Ok(number)
}

fn column_label(mut value: u32) -> String {
    let mut label = String::new();
    while value > 0 {
        let remainder = (value - 1) % 26;
        label.push(char::from(
            b'A' + u8::try_from(remainder).unwrap_or_default(),
        ));
        value = (value - 1) / 26;
    }
    label.chars().rev().collect()
}

fn resolve_axis(
    start: Option<u32>,
    end: Option<u32>,
    default: Option<u32>,
    label: &str,
) -> Result<(u32, u32), String> {
    let start = start.unwrap_or(1);
    let end = end.or(default).ok_or_else(|| {
        format!("A1 {label} bound is open and spreadsheet metadata has no dimension")
    })?;
    if start == 0 || end == 0 || start > end {
        return Err(format!("A1 {label} bounds are invalid"));
    }
    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::A1Range;

    #[test]
    fn parses_and_pages_a1_ranges_without_dropping_coordinates() {
        let range = A1Range::new("'Sheet 1'!A1:B10").unwrap();
        let bounds = range.bounds().unwrap();
        let bounds = bounds.resolve(Some(10), Some(5)).unwrap();

        assert_eq!(bounds.row_count(), 10);
        assert_eq!(bounds.column_count(), 2);
        assert_eq!(
            range.subrange(6, 1, 5, 2).unwrap().as_str(),
            "'Sheet 1'!A6:B10"
        );
        assert!(A1Range::new("a1").unwrap().is_single_cell());
    }
}
