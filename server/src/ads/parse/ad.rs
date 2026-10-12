//! Ad row from `/act_{id}/ads` (creative flattened for the drawer).

use serde_json::Value;

use crate::model::Ad;

use super::primitives::str_of;

pub fn parse_ad(raw: &Value, adset_id: &str) -> Option<Ad> {
    let id = str_of(raw, "id");
    if id.is_empty() {
        return None;
    }
    let creative = raw.get("creative").cloned().unwrap_or(Value::Null);
    Some(Ad {
        id: id.to_string(),
        adset_id: adset_id.to_string(),
        name: str_of(raw, "name").to_string(),
        status: str_of(raw, "status").to_string(),
        effective_status: str_of(raw, "effective_status").to_string(),
        creative_title: str_of(&creative, "title").to_string(),
        creative_body: str_of(&creative, "body").to_string(),
        image_url: str_of(&creative, "image_url").to_string(),
        thumbnail_url: str_of(&creative, "thumbnail_url").to_string(),
        story_id: {
            let story = str_of(&creative, "effective_object_story_id");
            if story.is_empty() {
                str_of(&creative, "object_story_id").to_string()
            } else {
                story.to_string()
            }
        },
        preview_url: {
            // A shareable preview is nicer than a raw image for the drawer.
            let thumb = str_of(&creative, "thumbnail_url");
            if thumb.is_empty() {
                str_of(&creative, "image_url").to_string()
            } else {
                thumb.to_string()
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ad_flattens_creative_and_prefers_effective_story() {
        let raw = json!({
            "id": "a1", "name": "Ad", "status": "PAUSED",
            "creative": {
                "title": "Hello", "body": "Body text",
                "image_url": "https://img.example/i.jpg",
                "thumbnail_url": "https://img.example/t.jpg",
                "object_story_id": "p1_1", "effective_object_story_id": "p1_2"
            }
        });
        let ad = parse_ad(&raw, "s1").unwrap();
        assert_eq!(ad.adset_id, "s1");
        assert_eq!(ad.creative_title, "Hello");
        assert_eq!(ad.story_id, "p1_2");
        assert_eq!(ad.preview_url, "https://img.example/t.jpg");
    }
}
