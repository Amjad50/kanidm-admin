use kanidm_proto::v1::Entry;

/// First value of an attr on a kanidm `Entry`. The entry's `attrs` field is a
/// flat `BTreeMap<String, Vec<String>>`; most consumers want the single first
/// string for a named attr.
pub fn attr_first(entry: &Entry, name: &str) -> Option<String> {
    entry.attrs.get(name).and_then(|v| v.first().cloned())
}

/// All values of an attr on a kanidm `Entry`. Returns an empty vec if the attr
/// is absent.
pub fn attr_all(entry: &Entry, name: &str) -> Vec<String> {
    entry
        .attrs
        .get(name)
        .cloned()
        .unwrap_or_default()
}

/// True if the attr is present (non-empty) on the entry.
pub fn attr_present(entry: &Entry, name: &str) -> bool {
    entry
        .attrs
        .get(name)
        .is_some_and(|v| !v.is_empty())
}

/// Returns the SPN of the entry, or falls back to UUID when SPN is absent.
/// Kanidm service accounts and some transitional entries lack an SPN;
/// UUID is always present and stable, so it makes a safe fallback key.
pub fn spn_or_uuid(entry: &Entry) -> String {
    attr_first(entry, "spn")
        .or_else(|| attr_first(entry, "uuid"))
        .unwrap_or_default()
}
