# Upstream provenance

Source project: https://github.com/serenity-dojo/cashback-rewards

Baseline used for the port:
- branch: `section-13/solution`
- commit: `1f8e0083cf61dbb1c852e3140b9fb56b8398cc0a`

The Rust implementation preserves the business rules and current production API surface of that baseline. The monthly cashback-report endpoint described only in the upstream OpenAPI document is intentionally not implemented because it is not present in the corresponding Java production code.
