use axum::http::HeaderValue;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToastKind {
    Success,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct Toast {
    pub title: String,
    pub kind: ToastKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
}

#[allow(dead_code)]
impl Toast {
    pub fn success(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            kind: ToastKind::Success,
            desc: None,
        }
    }
    pub fn info(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            kind: ToastKind::Info,
            desc: None,
        }
    }
    pub fn warn(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            kind: ToastKind::Warn,
            desc: None,
        }
    }
    pub fn error(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            kind: ToastKind::Error,
            desc: None,
        }
    }
    pub fn with_desc(mut self, desc: impl Into<String>) -> Self {
        self.desc = Some(desc.into());
        self
    }

    /// Build the `HX-Trigger` header value carrying this toast. HTMX parses
    /// the JSON and fires a `toast` event with the payload as `event.detail`.
    pub fn hx_trigger(&self) -> HeaderValue {
        #[derive(Serialize)]
        struct Envelope<'a> {
            toast: &'a Toast,
        }
        let json =
            serde_json::to_string(&Envelope { toast: self }).expect("Toast is always serialisable");
        HeaderValue::from_str(&json).expect("toast JSON is always valid HTTP header bytes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_serialises_with_lowercase_kind() {
        let t = Toast::success("Saved");
        let s = serde_json::to_string(&t).unwrap();
        assert!(s.contains(r#""kind":"success""#), "got: {s}");
        assert!(s.contains(r#""title":"Saved""#));
        assert!(!s.contains("desc"));
    }

    #[test]
    fn with_desc_includes_desc_field() {
        let t = Toast::warn("Heads up").with_desc("details here");
        let s = serde_json::to_string(&t).unwrap();
        assert!(s.contains(r#""kind":"warn""#));
        assert!(s.contains(r#""desc":"details here""#));
    }

    #[test]
    fn hx_trigger_wraps_in_toast_envelope() {
        let header = Toast::error("Boom").hx_trigger();
        let s = header.to_str().unwrap();
        let v: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(v["toast"]["title"], "Boom");
        assert_eq!(v["toast"]["kind"], "error");
    }
}
