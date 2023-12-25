use crate::color::Color;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockData {
    pub name: String,
    pub color: Color,
}
