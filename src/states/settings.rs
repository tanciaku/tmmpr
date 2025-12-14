
pub struct SettingsState {
    recent_paths: RecentPaths,
}

pub struct RecentPaths {
    pub recent_path_1: Option<String>,
    pub recent_path_2: Option<String>,
    pub recent_path_3: Option<String>,
}

// you read it first, modify select fields, and rewrite it back