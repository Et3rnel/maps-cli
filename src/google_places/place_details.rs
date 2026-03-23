use anyhow::{Context, Result};
use reqwest::blocking::Client;

use crate::google_places::text_search::Place;

const API_URL: &str = "https://places.googleapis.com/v1/places";
const REVIEWS_FIELD_MASK: &str = "reviews";

pub fn fetch_reviews(
    client: &Client,
    api_key: &str,
    place_id: &str,
) -> Result<Option<Vec<crate::google_places::text_search::Review>>> {
    let response = client
        .get(format!("{API_URL}/{place_id}"))
        .header("Content-Type", "application/json")
        .header("X-Goog-Api-Key", api_key)
        .header("X-Goog-FieldMask", REVIEWS_FIELD_MASK)
        .send()
        .with_context(|| format!("failed to call Place Details for {place_id}"))?
        .error_for_status()
        .with_context(|| format!("Place Details request failed for {place_id}"))?;

    let place: Place = response
        .json()
        .with_context(|| format!("failed to parse Place Details response for {place_id}"))?;

    Ok(place.reviews)
}
