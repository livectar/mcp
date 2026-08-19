use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct PageSize(u16);

impl PageSize {
    pub const MAX: u16 = 100;

    pub fn new(value: u16) -> Result<Self, String> {
        if !(1..=Self::MAX).contains(&value) {
            return Err(format!("page_size must be between 1 and {}", Self::MAX));
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for PageSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u16::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct CellLimit(u32);

impl CellLimit {
    pub const MAX: u32 = 10_000;

    pub fn new(value: u32) -> Result<Self, String> {
        if !(1..=Self::MAX).contains(&value) {
            return Err(format!("max_cells must be between 1 and {}", Self::MAX));
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct TextChunkSize(u32);

impl TextChunkSize {
    pub const MIN: u32 = 256;
    pub const MAX: u32 = 32 * 1024;

    pub fn new(value: u32) -> Result<Self, String> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(format!(
                "chunk_bytes must be between {} and {}",
                Self::MIN,
                Self::MAX
            ));
        }
        Ok(Self(value))
    }

    pub const fn default_value() -> Self {
        Self(8 * 1024)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl Default for TextChunkSize {
    fn default() -> Self {
        Self::default_value()
    }
}

impl<'de> Deserialize<'de> for TextChunkSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u32::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

impl<'de> Deserialize<'de> for CellLimit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u32::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::TextChunkSize;

    #[test]
    fn validates_text_chunk_size() {
        assert!(TextChunkSize::new(TextChunkSize::MIN - 1).is_err());
        assert!(TextChunkSize::new(TextChunkSize::MAX + 1).is_err());
        assert_eq!(TextChunkSize::default().get(), 8 * 1024);
    }
}
