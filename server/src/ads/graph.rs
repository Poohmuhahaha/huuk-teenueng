//! `GET`/`POST {GRAPH}{path}` helpers with Graph error mapping.

use serde_json::Value;

use crate::oauth::OauthSettings;

/// `GET {GRAPH}{path}` with the access token appended; Graph errors are mapped
/// to their message so the UI can show something actionable.
pub(crate) async fn graph_get(
    base: &str,
    oauth: &OauthSettings,
    path: &str,
    query: &[(&str, &str)],
    token: &str,
) -> Result<Value, String> {
    let res = oauth
        .http()
        .get(format!("{base}{path}"))
        .query(query)
        .query(&[("access_token", token)])
        .send()
        .await
        .map_err(|e| format!("graph request failed: {e}"))?;
    let status = res.status();
    let body: Value = res
        .json()
        .await
        .map_err(|e| format!("graph response was not JSON ({status}): {e}"))?;
    if let Some(err) = body.get("error") {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown Meta error");
        return Err(format!("graph error: {msg}"));
    }
    Ok(body)
}

/// `POST {GRAPH}{path}` with form fields + token. Returns the response body.
pub(crate) async fn graph_post(
    base: &str,
    oauth: &OauthSettings,
    path: &str,
    fields: &[(&str, String)],
    token: &str,
) -> Result<Value, String> {
    let form: Vec<(&str, &str)> = fields
        .iter()
        .map(|(k, v)| (*k, v.as_str()))
        .chain(std::iter::once(("access_token", token)))
        .collect();
    let res = oauth
        .http()
        .post(format!("{base}{path}"))
        .form(&form)
        .send()
        .await
        .map_err(|e| format!("graph request failed: {e}"))?;
    let status = res.status();
    let body: Value = res
        .json()
        .await
        .map_err(|e| format!("graph response was not JSON ({status}): {e}"))?;
    if let Some(err) = body.get("error") {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown Meta error");
        return Err(format!("graph error: {msg}"));
    }
    Ok(body)
}
