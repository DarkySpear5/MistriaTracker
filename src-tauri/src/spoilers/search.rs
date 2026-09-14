use super::policy::{AppSnapshot, VisibleItem};

pub struct SearchService;

impl SearchService {
    pub fn search(snapshot: &AppSnapshot, query: &str) -> Vec<VisibleItem> {
        let query = query.to_lowercase();
        snapshot
            .items
            .iter()
            .filter(|item| {
                item.name.to_lowercase().contains(&query)
                    || item.description.to_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    }
}
