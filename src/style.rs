#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Style {
    pub id: u16,

    /// 32-bit RGBA value
    pub fg_color: Option<u32>,

    /// 32-bit RGBA value, default 0
    pub bg_color: Option<u32>,

    /// 100..900, default 400
    pub weight: Option<u16>,

    /// default false
    pub italic: Option<bool>,
}
