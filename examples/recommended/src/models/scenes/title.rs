//! Title scene data

#[derive(Debug)]
pub struct TitleSceneData {
    pub selected_index: usize,
}

impl TitleSceneData {
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }
}

impl Default for TitleSceneData {
    fn default() -> Self {
        Self::new()
    }
}
