# Tech Stack — content-planner-huuk (อ่านจากไฟล์จริง)

- Frontend `app/package.json`: `vue@3.5.42`, `vue-router@5.3.1`, `@tanstack/vue-query@5.102.8`, `@tanstack/vue-table@9.2.4`, `motion-v@2.4.2` | dev: `vite@8.3.0`, `vitest@5.0.0`, `vue-tsc@3.3.11`, `typescript@6.0.2`, `@tauri-apps/cli@2`
- Backend `server/Cargo.toml`: `axum@0.8`, `tokio@1`, `serde@1`, `serde_json@1`, `argon2@0.5`, `chacha20poly1305@0.10`, `blake2@0.10`, `reqwest@0.13.5`, `tower-http@0.6`, `chrono@0.4`
- Lock: `app/bun.lock` + `server/Cargo.lock` (ตรวจ `--frozen-lockfile` ผ่าน 2026-09-27)
- Deploy: binary เสิร์ฟ API+frontend (`:8787`), nginx (compose), Tauri desktop, CI ใช้ `oven-sh/setup-bun`

## Lock แล้ว
- Package manager: **Bun เท่านั้น** — ห้าม `package-lock.json` (CI พังถ้ามีซ้ำ)
- Ports: backend `8787`, frontend dev `5173` | Auth: Argon2id + session tokens + rate-limit login
