use nucleo::pattern::{CaseMatching, Normalization, Pattern};
use nucleo::{Config, Matcher as NucleoMatcher, Utf32String};

pub struct Matcher {
    items: Vec<Utf32String>,
    inner: NucleoMatcher,
    query: String,
}

impl Matcher {
    pub fn new(items: Vec<String>) -> Self {
        let items = items.into_iter().map(Utf32String::from).collect();
        Self {
            items,
            inner: NucleoMatcher::new(Config::DEFAULT),
            query: String::new(),
        }
    }

    pub fn set_query(&mut self, q: &str) {
        self.query = q.to_string();
    }

    pub fn matches(&mut self) -> Vec<usize> {
        if self.query.is_empty() {
            return (0..self.items.len()).collect();
        }
        let pattern = Pattern::parse(&self.query, CaseMatching::Smart, Normalization::Smart);
        let mut scored: Vec<(usize, u32)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                pattern
                    .score(s.slice(..), &mut self.inner)
                    .map(|score| (i, score))
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        scored.into_iter().map(|(i, _)| i).collect()
    }
}
