use crate::models::{PasswordVault, PasswordEntry};

impl PasswordVault {
  pub fn search_entries(&self, query: &str) -> Vec<(usize, &PasswordEntry, u32)> {
    let query = query.to_lowercase();

    self.entries
      .iter()
      .enumerate()
      .filter_map(|(i, entry)| {
        if query.is_empty() {
          return Some((i, entry, 0));
        }

        let service_lower = entry.service.to_lowercase();
        let user_lower = entry.username.to_lowercase();

        let mut score = 0;
        if service_lower == query || user_lower == query { score += 1000; }
        if service_lower.starts_with(&query) { score += 100; }
        if user_lower.starts_with(&query) { score += 80; }
        if service_lower.contains(&query) { score += 10; }
        if user_lower.contains(&query) { score += 5; }

        if score > 0 { Some((i, entry, score)) } else { None }
      })
      .collect::<Vec<_>>()
  }
}
