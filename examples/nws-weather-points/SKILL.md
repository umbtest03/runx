---
name: nws-weather-points
description: Governed HTTP GET against the public National Weather Service points endpoint.
source:
  type: http
  url: https://api.weather.gov/points/{lat},{lon}
  method: GET
  headers:
    user-agent: "runx-weather-demo/0.1 (https://github.com/runxhq/runx)"
    accept: "application/geo+json, application/json"
inputs:
  lat:
    type: string
    required: true
    description: Latitude with no more than four decimal places.
  lon:
    type: string
    required: true
    description: Longitude with no more than four decimal places.
---
A real public-provider proof for the first-class `http` front. This skill calls
the National Weather Service `points/{lat},{lon}` endpoint, which is part of the
official `api.weather.gov` OpenAPI surface, through the governed Runx HTTP
transport. It has no API key and no private-network opt-in.

The static `User-Agent` header is intentional: `api.weather.gov` requires callers
to identify themselves. The graph example validates stable provider metadata and
forecast URLs rather than volatile forecast prose.
