// Server-side pagination state. The actual buttons are rendered client-side
// by `islands/pagination.tsx` — this struct just carries the values that the
// island reads from `data-*` attributes on its mount host.

pub struct Pagination {
    /// 1-indexed current page.
    pub page: usize,
    pub total_pages: usize,
    pub filtered_count: usize,
    pub per_page: usize,
    /// Base URL for the list (e.g. `/people`). The island appends `?page=N` and
    /// preserves any filter params already on `window.location`.
    pub base_url: &'static str,
    /// HTMX swap target for the rows region (e.g. `#people-tbody`).
    pub target: &'static str,
}
