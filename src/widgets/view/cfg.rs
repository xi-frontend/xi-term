pub struct ViewConfig {
    // Gutter Settings
    pub display_gutter: bool,
    pub gutter_size: u16,
    // Tab Settings
    pub tab_size: u16
}

impl Default for ViewConfig {
    fn default() ->ViewConfig {
        ViewConfig {
            display_gutter: true,
            gutter_size: 0,
            tab_size: 4,
        }
    }
}
