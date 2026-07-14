use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub search: Option<String>,
    pub sort: Option<String>,
}

#[allow(dead_code)]
impl PaginationParams {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn limit(&self) -> u64 {
        self.limit.unwrap_or(20).clamp(1, 100)
    }

    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.limit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_defaults_and_clamping() {
        let p = PaginationParams {
            page: None,
            limit: None,
            search: None,
            sort: None,
        };
        assert_eq!(p.page(), 1);
        assert_eq!(p.limit(), 20);
        assert_eq!(p.offset(), 0);

        let p_custom = PaginationParams {
            page: Some(3),
            limit: Some(500), // Exceeds max 100
            search: None,
            sort: None,
        };
        assert_eq!(p_custom.page(), 3);
        assert_eq!(p_custom.limit(), 100);
        assert_eq!(p_custom.offset(), 200);
    }
}
