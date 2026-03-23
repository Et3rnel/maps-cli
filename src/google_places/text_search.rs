use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const API_URL: &str = "https://places.googleapis.com/v1/places:searchText";
const DEFAULT_FIELD_MASK: &str = "places.displayName,places.formattedAddress,places.rating,places.userRatingCount,places.priceLevel,places.types,places.websiteUri,nextPageToken";

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextSearchResponse {
    #[serde(default)]
    pub places: Vec<Place>,
    pub next_page_token: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub id: Option<String>,
    pub display_name: Option<DisplayName>,
    pub formatted_address: Option<String>,
    pub rating: Option<f64>,
    pub user_rating_count: Option<u32>,
    pub types: Option<Vec<String>>,
    pub website_uri: Option<String>,
    pub price_level: Option<String>,
    pub reviews: Option<Vec<Review>>,
    #[serde(default)]
    pub reviews_fetched: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DisplayName {
    pub text: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub rating: Option<f64>,
    pub relative_publish_time_description: Option<String>,
    pub original_text: Option<LocalizedText>,
    pub author_attribution: Option<AuthorAttribution>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocalizedText {
    pub text: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthorAttribution {
    pub display_name: Option<String>,
}

pub fn fetch(
    client: &Client,
    api_key: &str,
    body: Value,
    field_mask: &str,
) -> Result<TextSearchResponse> {
    client
        .post(API_URL)
        .header("Content-Type", "application/json")
        .header("X-Goog-Api-Key", api_key)
        .header("X-Goog-FieldMask", field_mask)
        .json(&body)
        .send()
        .context("failed to call Google Places API")?
        .error_for_status()
        .context("Google Places API request failed")?
        .json()
        .context("failed to parse response")
}

pub fn default_field_mask() -> &'static str {
    DEFAULT_FIELD_MASK
}

pub struct TextSearchParams<'a> {
    pub query: &'a str,
    pub language: &'a str,
    pub page_size: Option<u8>,
    pub included_type: Option<&'a str>,
    pub open_now: bool,
    pub min_rating: Option<f64>,
    pub price_levels: Option<&'a [String]>,
    pub rank_preference: Option<&'a str>,
    pub region_code: Option<&'a str>,
    pub location_bias: Option<&'a str>,
    pub page_token: Option<&'a str>,
}

pub fn build_request_body(params: TextSearchParams<'_>) -> Value {
    let mut body = json!({
        "textQuery": params.query,
        "languageCode": params.language,
    });

    let obj = body.as_object_mut().unwrap();

    if let Some(page_size) = params.page_size {
        obj.insert("pageSize".to_string(), json!(page_size));
    }

    if let Some(included_type) = params.included_type {
        obj.insert("includedType".to_string(), json!(included_type));
    }

    if params.open_now {
        obj.insert("openNow".to_string(), json!(true));
    }

    if let Some(min_rating) = params.min_rating {
        obj.insert("minRating".to_string(), json!(min_rating));
    }

    if let Some(price_levels) = params.price_levels {
        obj.insert("priceLevels".to_string(), json!(price_levels));
    }

    if let Some(rank_preference) = params.rank_preference {
        obj.insert("rankPreference".to_string(), json!(rank_preference));
    }

    if let Some(region_code) = params.region_code {
        obj.insert("regionCode".to_string(), json!(region_code));
    }

    if let Some(location_bias) = params.location_bias
        && let Some(circle) = parse_location_bias(location_bias)
    {
        obj.insert("locationBias".to_string(), circle);
    }

    if let Some(page_token) = params.page_token {
        obj.insert("pageToken".to_string(), json!(page_token));
    }

    body
}

pub fn parse_location_bias(input: &str) -> Option<Value> {
    let parts: Vec<&str> = input.split(',').collect();
    if parts.len() != 3 {
        eprintln!("warning: --location-bias must be lat,lng,radius (e.g. 48.8566,2.3522,500)");
        return None;
    }

    let lat: f64 = parts[0].trim().parse().ok()?;
    let lng: f64 = parts[1].trim().parse().ok()?;
    let radius: f64 = parts[2].trim().parse().ok()?;

    Some(json!({
        "circle": {
            "center": {
                "latitude": lat,
                "longitude": lng,
            },
            "radius": radius,
        }
    }))
}
