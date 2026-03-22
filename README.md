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

# Limit results and get raw JSON
maps-cli text-search "coffee in London" --page-size 5 --json

# Filter by price level
maps-cli text-search "restaurant in Tokyo" --price-levels PRICE_LEVEL_INEXPENSIVE,PRICE_LEVEL_MODERATE

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
| `--json` | Output raw JSON |
