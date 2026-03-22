use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{Value, json};

const API_URL: &str = "https://places.googleapis.com/v1/places:searchText";

const DEFAULT_FIELD_MASK: &str = "places.displayName,places.formattedAddress,places.rating,places.userRatingCount,places.priceLevel,places.types,places.websiteUri,nextPageToken";

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Text query to search for (e.g. "pizza in New York")
    #[arg(value_name = "QUERY")]
    pub query: String,

    /// Language code for results (e.g. "fr", "en")
    #[arg(long, default_value = "en")]
    pub language: String,

    /// Maximum number of results per page (1-20)
    #[arg(long)]
    pub page_size: Option<u8>,

    /// Filter by place type (e.g. "restaurant", "bar")
    #[arg(long)]
    pub included_type: Option<String>,

    /// Only return places that are currently open
    #[arg(long, default_value_t = false)]
    pub open_now: bool,

    /// Minimum user rating (0.0-5.0, increments of 0.5)
    #[arg(long)]
    pub min_rating: Option<f64>,

    /// Price levels to include (e.g. PRICE_LEVEL_MODERATE)
    #[arg(long, value_delimiter = ',')]
    pub price_levels: Option<Vec<String>>,

    /// Rank results by RELEVANCE or DISTANCE
    #[arg(long)]
    pub rank_preference: Option<String>,

    /// Region code for formatting (e.g. "us", "fr")
    #[arg(long)]
    pub region_code: Option<String>,

    /// Location bias as a circle: lat,lng,radius (e.g. "48.8566,2.3522,500")
    #[arg(long)]
    pub location_bias: Option<String>,

    /// Custom field mask (comma-separated, overrides default)
    #[arg(long)]
    pub fields: Option<String>,

    /// Page token for pagination (from a previous response)
    #[arg(long)]
    pub page_token: Option<String>,

    /// Output raw JSON instead of formatted text
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TextSearchResponse {
    #[serde(default)]
    places: Vec<Place>,
    next_page_token: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Place {
    display_name: Option<DisplayName>,
    formatted_address: Option<String>,
    rating: Option<f64>,
    user_rating_count: Option<u32>,
    types: Option<Vec<String>>,
    website_uri: Option<String>,
    price_level: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DisplayName {
    text: String,
}

pub fn run(api_key: &str, args: &Args) -> Result<()> {
    let client = Client::builder()
        .build()
        .context("failed to build HTTP client")?;

    let body = build_request_body(args);
    let field_mask = args.fields.as_deref().unwrap_or(DEFAULT_FIELD_MASK);

    let response = client
        .post(API_URL)
        .header("Content-Type", "application/json")
        .header("X-Goog-Api-Key", api_key)
        .header("X-Goog-FieldMask", field_mask)
        .json(&body)
        .send()
        .context("failed to call Google Places API")?
        .error_for_status()
        .context("Google Places API request failed")?;

    if args.json {
        let raw: Value = response.json().context("failed to parse response")?;
        println!("{}", serde_json::to_string_pretty(&raw)?);
    } else {
        let result: TextSearchResponse = response.json().context("failed to parse response")?;
        render_output(&result);
    }

    Ok(())
}

fn build_request_body(args: &Args) -> Value {
    let mut body = json!({
        "textQuery": args.query,
        "languageCode": args.language,
    });

    let obj = body.as_object_mut().unwrap();

    if let Some(page_size) = args.page_size {
        obj.insert("pageSize".to_string(), json!(page_size));
    }

    if let Some(ref included_type) = args.included_type {
        obj.insert("includedType".to_string(), json!(included_type));
    }

    if args.open_now {
        obj.insert("openNow".to_string(), json!(true));
    }

    if let Some(min_rating) = args.min_rating {
        obj.insert("minRating".to_string(), json!(min_rating));
    }

    if let Some(ref price_levels) = args.price_levels {
        obj.insert("priceLevels".to_string(), json!(price_levels));
    }

    if let Some(ref rank_preference) = args.rank_preference {
        obj.insert("rankPreference".to_string(), json!(rank_preference));
    }

    if let Some(ref region_code) = args.region_code {
        obj.insert("regionCode".to_string(), json!(region_code));
    }

    if let Some(ref location_bias) = args.location_bias
        && let Some(circle) = parse_location_bias(location_bias)
    {
        obj.insert("locationBias".to_string(), circle);
    }

    if let Some(ref page_token) = args.page_token {
        obj.insert("pageToken".to_string(), json!(page_token));
    }

    body
}

fn parse_location_bias(input: &str) -> Option<Value> {
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

fn render_output(result: &TextSearchResponse) {
    if result.places.is_empty() {
        println!("No places found.");
        return;
    }

    for (i, place) in result.places.iter().enumerate() {
        let name = place
            .display_name
            .as_ref()
            .map(|d| d.text.as_str())
            .unwrap_or("(unknown)");

        println!("{}. {}", i + 1, name);

        if let Some(ref address) = place.formatted_address {
            println!("   Address: {address}");
        }

        if let Some(rating) = place.rating {
            let count = place.user_rating_count.unwrap_or(0);
            println!("   Rating:  {rating}/5 ({count} reviews)");
        }

        if let Some(ref price) = place.price_level {
            let display = price.strip_prefix("PRICE_LEVEL_").unwrap_or(price);
            println!("   Price:   {display}");
        }

        if let Some(ref types) = place.types {
            let display: Vec<&str> = types.iter().take(5).map(|t| t.as_str()).collect();
            println!("   Types:   {}", display.join(", "));
        }

        if let Some(ref uri) = place.website_uri {
            println!("   Website: {uri}");
        }

        if i < result.places.len() - 1 {
            println!();
        }
    }

    if let Some(ref token) = result.next_page_token {
        println!("\n--- More results available. Use --page-token '{token}' to see the next page.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_body_minimal() {
        let args = Args {
            query: "pizza in Paris".to_string(),
            language: "fr".to_string(),
            page_size: None,
            included_type: None,
            open_now: false,
            min_rating: None,
            price_levels: None,
            rank_preference: None,
            region_code: None,
            location_bias: None,
            fields: None,
            page_token: None,
            json: false,
        };

        let body = build_request_body(&args);
        assert_eq!(body["textQuery"], "pizza in Paris");
        assert_eq!(body["languageCode"], "fr");
        assert!(body.get("pageSize").is_none());
        assert!(body.get("openNow").is_none());
    }

    #[test]
    fn build_request_body_with_options() {
        let args = Args {
            query: "restaurant".to_string(),
            language: "en".to_string(),
            page_size: Some(5),
            included_type: Some("restaurant".to_string()),
            open_now: true,
            min_rating: Some(4.0),
            price_levels: Some(vec!["PRICE_LEVEL_MODERATE".to_string()]),
            rank_preference: Some("RELEVANCE".to_string()),
            region_code: Some("us".to_string()),
            location_bias: None,
            fields: None,
            page_token: None,
            json: false,
        };

        let body = build_request_body(&args);
        assert_eq!(body["pageSize"], 5);
        assert_eq!(body["includedType"], "restaurant");
        assert_eq!(body["openNow"], true);
        assert_eq!(body["minRating"], 4.0);
        assert_eq!(body["rankPreference"], "RELEVANCE");
        assert_eq!(body["regionCode"], "us");
    }

    #[test]
    fn parse_location_bias_valid() {
        let result = parse_location_bias("48.8566,2.3522,500").unwrap();
        assert_eq!(result["circle"]["center"]["latitude"], 48.8566);
        assert_eq!(result["circle"]["center"]["longitude"], 2.3522);
        assert_eq!(result["circle"]["radius"], 500.0);
    }

    #[test]
    fn parse_location_bias_invalid() {
        assert!(parse_location_bias("invalid").is_none());
        assert!(parse_location_bias("48.8,2.3").is_none());
    }

    #[test]
    fn render_output_empty() {
        let result = TextSearchResponse {
            places: vec![],
            next_page_token: None,
        };
        // Should not panic
        render_output(&result);
    }
}
