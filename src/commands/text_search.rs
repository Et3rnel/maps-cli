use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use reqwest::blocking::Client;

use crate::google_places::{place_details, text_search as api};

const DEFAULT_REVIEWS_TOP: usize = 5;

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

    /// Fetch Google reviews for the first results
    #[arg(long, visible_alias = "with-reviews", default_value_t = false)]
    pub reviews: bool,

    /// Limit review lookups to the first N places (default: 5)
    #[arg(long, requires = "reviews", value_name = "N", value_parser = parse_reviews_top)]
    pub reviews_top: Option<usize>,

    /// Only fetch reviews for places with rating >= this value
    #[arg(long, requires = "reviews")]
    pub reviews_min_rating: Option<f64>,

    /// Only fetch reviews for places with at least this many ratings
    #[arg(long, requires = "reviews")]
    pub reviews_min_count: Option<u32>,
}

pub fn run(api_key: &str, args: &Args) -> Result<()> {
    let client = Client::builder()
        .build()
        .context("failed to build HTTP client")?;

    let body = api::build_request_body(api::TextSearchParams {
        query: &args.query,
        language: &args.language,
        page_size: args.page_size,
        included_type: args.included_type.as_deref(),
        open_now: args.open_now,
        min_rating: args.min_rating,
        price_levels: args.price_levels.as_deref(),
        rank_preference: args.rank_preference.as_deref(),
        region_code: args.region_code.as_deref(),
        location_bias: args.location_bias.as_deref(),
        page_token: args.page_token.as_deref(),
    });

    let field_mask = build_search_field_mask(args);
    let mut result = api::fetch(&client, api_key, body, &field_mask)?;

    if args.reviews {
        enrich_places_with_reviews(&client, api_key, args, &mut result.places);
    }

    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

fn build_search_field_mask(args: &Args) -> String {
    match args.fields.as_deref() {
        Some(field_mask) if args.reviews => ensure_field_in_mask(field_mask, "places.id"),
        Some(field_mask) => field_mask.to_string(),
        None if args.reviews => ensure_field_in_mask(api::default_field_mask(), "places.id"),
        None => api::default_field_mask().to_string(),
    }
}

fn ensure_field_in_mask(field_mask: &str, required_field: &str) -> String {
    let has_field = field_mask
        .split(',')
        .map(str::trim)
        .any(|field| field == required_field);

    if has_field {
        field_mask.to_string()
    } else {
        format!("{field_mask},{required_field}")
    }
}

fn parse_reviews_top(input: &str) -> Result<usize, String> {
    let value = input
        .parse::<usize>()
        .map_err(|_| format!("invalid value '{input}': expected a positive integer"))?;

    if value == 0 {
        Err("reviews top must be greater than 0".to_string())
    } else {
        Ok(value)
    }
}

fn enrich_places_with_reviews(
    client: &Client,
    api_key: &str,
    args: &Args,
    places: &mut [api::Place],
) {
    let limit = args
        .reviews_top
        .unwrap_or(DEFAULT_REVIEWS_TOP)
        .min(places.len());

    for place in places
        .iter_mut()
        .filter(|place| should_fetch_reviews(place, args))
        .take(limit)
    {
        let Some(place_id) = place.id.as_deref() else {
            eprintln!("warning: skipping review lookup for a result without a place id");
            continue;
        };

        match place_details::fetch_reviews(client, api_key, place_id) {
            Ok(reviews) => {
                place.reviews = reviews;
                place.reviews_fetched = true;
            }
            Err(err) => {
                let name = place
                    .display_name
                    .as_ref()
                    .map(|display_name| display_name.text.as_str())
                    .unwrap_or(place_id);
                eprintln!("warning: failed to fetch reviews for {name}: {err:#}");
            }
        }
    }
}

fn should_fetch_reviews(place: &api::Place, args: &Args) -> bool {
    if let Some(min_rating) = args.reviews_min_rating
        && place.rating.is_none_or(|rating| rating < min_rating)
    {
        return false;
    }

    if let Some(min_count) = args.reviews_min_count
        && place
            .user_rating_count
            .is_none_or(|count| count < min_count)
    {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_args() -> Args {
        Args {
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
            reviews: false,
            reviews_top: None,
            reviews_min_rating: None,
            reviews_min_count: None,
        }
    }

    #[test]
    fn build_request_body_minimal() {
        let args = make_args();
        let body = api::build_request_body(api::TextSearchParams {
            query: &args.query,
            language: &args.language,
            page_size: args.page_size,
            included_type: args.included_type.as_deref(),
            open_now: args.open_now,
            min_rating: args.min_rating,
            price_levels: args.price_levels.as_deref(),
            rank_preference: args.rank_preference.as_deref(),
            region_code: args.region_code.as_deref(),
            location_bias: args.location_bias.as_deref(),
            page_token: args.page_token.as_deref(),
        });

        assert_eq!(body["textQuery"], "pizza in Paris");
        assert_eq!(body["languageCode"], "fr");
        assert!(body.get("pageSize").is_none());
        assert!(body.get("openNow").is_none());
    }

    #[test]
    fn build_request_body_with_options() {
        let mut args = make_args();
        args.query = "restaurant".to_string();
        args.language = "en".to_string();
        args.page_size = Some(5);
        args.included_type = Some("restaurant".to_string());
        args.open_now = true;
        args.min_rating = Some(4.0);
        args.price_levels = Some(vec!["PRICE_LEVEL_MODERATE".to_string()]);
        args.rank_preference = Some("RELEVANCE".to_string());
        args.region_code = Some("us".to_string());

        let body = api::build_request_body(api::TextSearchParams {
            query: &args.query,
            language: &args.language,
            page_size: args.page_size,
            included_type: args.included_type.as_deref(),
            open_now: args.open_now,
            min_rating: args.min_rating,
            price_levels: args.price_levels.as_deref(),
            rank_preference: args.rank_preference.as_deref(),
            region_code: args.region_code.as_deref(),
            location_bias: args.location_bias.as_deref(),
            page_token: args.page_token.as_deref(),
        });

        assert_eq!(body["pageSize"], 5);
        assert_eq!(body["includedType"], "restaurant");
        assert_eq!(body["openNow"], true);
        assert_eq!(body["minRating"], 4.0);
        assert_eq!(body["rankPreference"], "RELEVANCE");
        assert_eq!(body["regionCode"], "us");
    }

    #[test]
    fn parse_location_bias_valid() {
        let result = api::parse_location_bias("48.8566,2.3522,500").unwrap();
        assert_eq!(result["circle"]["center"]["latitude"], 48.8566);
        assert_eq!(result["circle"]["center"]["longitude"], 2.3522);
        assert_eq!(result["circle"]["radius"], 500.0);
    }

    #[test]
    fn parse_location_bias_invalid() {
        assert!(api::parse_location_bias("invalid").is_none());
        assert!(api::parse_location_bias("48.8,2.3").is_none());
    }

    #[test]
    fn build_search_field_mask_adds_place_id_for_reviews() {
        let mut args = make_args();
        args.reviews = true;

        let field_mask = build_search_field_mask(&args);
        assert!(field_mask.contains("places.id"));
    }

    #[test]
    fn ensure_field_in_mask_does_not_duplicate_existing_field() {
        let field_mask = ensure_field_in_mask("places.displayName,places.id", "places.id");
        assert_eq!(field_mask, "places.displayName,places.id");
    }

    #[test]
    fn should_fetch_reviews_respects_min_rating() {
        let mut args = make_args();
        args.reviews_min_rating = Some(4.4);

        let place = api::Place {
            id: Some("abc".to_string()),
            display_name: None,
            formatted_address: None,
            rating: Some(4.3),
            user_rating_count: Some(100),
            types: None,
            website_uri: None,
            price_level: None,
            reviews: None,
            reviews_fetched: false,
        };

        assert!(!should_fetch_reviews(&place, &args));
    }

    #[test]
    fn should_fetch_reviews_respects_min_count() {
        let mut args = make_args();
        args.reviews_min_count = Some(30);

        let place = api::Place {
            id: Some("abc".to_string()),
            display_name: None,
            formatted_address: None,
            rating: Some(4.8),
            user_rating_count: Some(12),
            types: None,
            website_uri: None,
            price_level: None,
            reviews: None,
            reviews_fetched: false,
        };

        assert!(!should_fetch_reviews(&place, &args));
    }

    #[test]
    fn should_fetch_reviews_allows_place_matching_thresholds() {
        let mut args = make_args();
        args.reviews_min_rating = Some(4.4);
        args.reviews_min_count = Some(30);

        let place = api::Place {
            id: Some("abc".to_string()),
            display_name: None,
            formatted_address: None,
            rating: Some(4.8),
            user_rating_count: Some(120),
            types: None,
            website_uri: None,
            price_level: None,
            reviews: None,
            reviews_fetched: false,
        };

        assert!(should_fetch_reviews(&place, &args));
    }
}
