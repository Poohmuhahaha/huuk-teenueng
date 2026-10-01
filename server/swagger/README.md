# Vendored Swagger UI assets

These files are the [Swagger UI](https://swagger.io/tools/swagger-ui/) distribution
(`swagger-ui-dist`), vendored so the API docs work offline and without a CDN.

| File | Source |
|---|---|
| `swagger-ui.css`, `swagger-ui-bundle.js` | https://unpkg.com/swagger-ui-dist@5.30.3/ |
| `LICENSE` | Apache-2.0, from the same package |

`index.html` and `swagger-init.js` are ours; they load the OpenAPI document from
`/api/openapi.yaml` and are served by the Rust server at `/api/docs`.

Upgrade procedure: bump the version in the URLs above, re-download both files, and
re-run `cargo test` (the docs routes are covered by `tests/api.rs`).
