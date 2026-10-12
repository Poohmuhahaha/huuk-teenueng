//! Permission errors become reconnect guidance (the UI shows this text).
//! Anything else passes through untouched so rate limits and outages keep
//! their original message.

pub fn permission_hint(error: &str) -> Option<String> {
    let lower = error.to_lowercase();
    if lower.contains("ads_read")
        || lower.contains("ads_management")
        || lower.contains("permission")
        || lower.contains("does not have permission")
    {
        return Some("reconnect Meta to grant ads access".into());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_errors_become_reconnect_guidance() {
        let error = "graph error: (#200) Requires ads_read permission";
        assert_eq!(
            permission_hint(error).as_deref(),
            Some("reconnect Meta to grant ads access")
        );
        assert!(permission_hint("graph error: rate limit").is_none());
    }
}
