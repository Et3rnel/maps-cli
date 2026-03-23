# maps-cli

CLI for Google Maps APIs (Places, Text Search, and more).

## Installation

```bash
cargo install --path .
```

## Usage

Set your Google Maps API key:

```bash
export GOOGLE_MAPS_API_KEY="your-api-key"
```

### Text search

Search for places using a text query:

```bash
maps-cli text-search "pizza in Paris"
```

With options:

```bash
# French results, only open places, near a location
maps-cli text-search "restaurant" --language fr --open-now --location-bias "48.8566,2.3522,500"

# Filter by type and minimum rating
maps-cli text-search "bar in New York" --included-type bar --min-rating 4.0

# Limit results
maps-cli text-search "coffee in London" --page-size 5

# Filter by price level
maps-cli text-search "restaurant in Tokyo" --price-levels PRICE_LEVEL_INEXPENSIVE,PRICE_LEVEL_MODERATE

# Enrich the first 5 results with Google reviews
maps-cli text-search "dentist in Paris" --reviews

# Enrich only the first 3 results with Google reviews
maps-cli text-search "dentist in Paris" --reviews --reviews-top 3

# Only fetch reviews for strong candidates
maps-cli text-search "dentist in Paris" --reviews --reviews-min-rating 4.4 --reviews-min-count 30

# Pagination
maps-cli text-search "pizza in New York" --page-size 5
# Then use the token from the output:
maps-cli text-search "pizza in New York" --page-size 5 --page-token "TOKEN_FROM_PREVIOUS_RESPONSE"
```

### Options

| Flag | Description |
|---|---|
| `--api-key` | Google Maps API key (or set `GOOGLE_MAPS_API_KEY`) |
| `--language` | Language code for results (default: `en`) |
| `--page-size` | Number of results per page (1-20) |
| `--included-type` | Filter by place type (e.g. `restaurant`, `bar`) |
| `--open-now` | Only return currently open places |
| `--min-rating` | Minimum user rating (0.0-5.0) |
| `--price-levels` | Comma-separated price levels |
| `--rank-preference` | `RELEVANCE` or `DISTANCE` |
| `--region-code` | Region code for formatting (e.g. `us`, `fr`) |
| `--location-bias` | Bias results to a circle: `lat,lng,radius` |
| `--fields` | Custom field mask (overrides default) |
| `--page-token` | Pagination token from previous response |
| `--reviews`, `--with-reviews` | Fetch up to 5 Google reviews per place via Place Details |
| `--reviews-top` | Limit review lookups to the first `N` places (default: `5`) |
| `--reviews-min-rating` | Only fetch reviews for places with rating >= this value |
| `--reviews-min-count` | Only fetch reviews for places with at least this many ratings |

### Reviews

- `--reviews` triggers extra Place Details calls only for the first results selected by `--reviews-top` and matching any review thresholds.
- Google Places returns at most 5 reviews per place, sorted by relevance on the new Places API.
- The CLI always returns JSON.
- For reviews, the CLI keeps `originalText` and does not expose the localized `text` field.
