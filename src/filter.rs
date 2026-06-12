pub enum FilterMode {
    BlackList,
    WhiteList,
}

pub struct Filter {
    pub list: Vec<String>,
    pub mode: FilterMode,
}

impl Filter {
    pub fn new(list: Vec<String>, mode: FilterMode) -> Self {
        Filter { list, mode }
    }

    pub fn should_mute(&self, _app_name: &str) -> bool {
        false
    }
}
