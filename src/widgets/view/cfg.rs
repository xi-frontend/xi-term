pub struct ViewConfig {
    // Gutter Settings
    pub gutter_size: u16,

    // Tab Settings
    pub tab_size: u16
}

impl Default for ViewConfig {
    fn default() ->ViewConfig {
        ViewConfig {
            gutter_size: 0,
            tab_size: 4,
        }
    }
}
