# API — content-planner-huuk (Axum, prefix `/api`, spec `server/openapi.yaml`, docs `/api/docs`)

- Health: `GET /api/health`, `GET /api/ready`, `GET|POST /api/setup*`
- Auth: `POST /api/auth/register|login|logout|logout-all|change-password|plan`, `GET /api/auth/me`, `PATCH /api/auth/profile`, `GET|POST|DELETE /api/auth/accounts[/{email}]`
- CMS: `GET|POST /api/content`, `GET|PATCH|DELETE /api/content/{id}`, `POST /api/content/{id}/publish|unpublish|schedule|duplicate`, `GET /api/content/{id}/revisions*`, `GET /api/public/content[/{slug}]`
- Planner: `GET|POST /api/posts`, `PATCH|DELETE /api/posts/{id}`, `POST /api/posts/{id}/lock|unlock`, `GET|POST /api/ideas`, `POST /api/ideas/{id}/toggle|promote`, `GET|POST /api/tags[/{id}]`, `GET|POST /api/metrics[/import]`
- Ads/brand: `GET /api/live`, `GET|POST /api/ads[/sync|manage|boost]`, `POST /api/ads/campaigns/{id}/status|budget|duplicate`, `GET|POST /api/txns`, `GET|PATCH /api/brand`, `GET|POST /api/campaigns`, `POST|DELETE /api/brand/fonts|images[/{name}]`
- Misc: `GET /api/workspaces*`, `GET|POST /api/platforms*`, `GET|POST /api/oauth/{platform}[/start|callback|pending|choose]`

Maintenance: เพิ่ม endpoint → ใส่ `server/openapi.yaml` + `app/src/api/contract.ts` + test ใน `server/tests/` พร้อมกัน
