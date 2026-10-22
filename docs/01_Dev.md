# Dev — content-planner-huuk

## Quick start (Bun เท่านั้น)
```sh
bun run dev        # = bash ops/scripts/dev.sh → Rust :8787 (DEMO_MODE=1) + Vite :5173, รอ /api/health ก่อน
bun run dev:mock   # frontend อย่างเดียว (mock ในหน่วยความจำ, ไม่ต้องมี Rust)
bun run dev:server # backend อย่างเดียว
bun run test       # cargo test + vitest
bun run build      # binary release + vue-tsc -b && vite build
```
Manual: `cd server && cargo run` แล้ว `cd app && echo 'VITE_API_URL=http://localhost:8787' > .env.local && bun run dev`

## โครง (หลังย้าย app/server)
```
content-planner-huuk/
├── app/        # Vue 3 + TanStack Query (api/index.ts สลับ mock/http อัตโนมัติ)
│   ├── src/app/{App.vue,main.ts,router/index.ts,style.css}
│   ├── src/pages/*.vue (20 หน้า)  src/components/{ui,layout,overlays,tables,editor,live}
│   └── src/core/{theme,queries,auth,session,oauth,screens}.ts  src/mock/{api,db}.ts
├── server/     # Rust Axum: main.rs lib.rs handlers.rs store.rs model.rs oauth.rs live.rs ads.rs
├── desktop/    # Tauri desktop shell (src/, icons/, Cargo.toml, tauri.conf.json)
├── ops/        # docker-compose.yml, nginx/, deploy/, scripts/{dev,deploy,install,backup,restore,smoke}.sh
├── docs/       # 00_STATUS..06_Business + DEPLOYMENT.md + 07_Plan.* + prompts/
├── Makefile  package.json (orchestrator)
└── data/ (live JSON, git-ignored)
```

## Deploy
- ไม่ใช้ Docker: `sudo ops/scripts/install.sh` → `http://localhost:8787` (config `/etc/content-planner.env`)
- Docker: `ops/scripts/deploy.sh init | up [--tls] | smoke | backup | restore | update` (ดู `docs/DEPLOYMENT.md`)

## Maintenance
- Lock เดียว: `app/bun.lock` + `server/Cargo.lock` — ห้ามเอา `package-lock.json` กลับมา (CI ใช้ `bun install --frozen-lockfile`)
- เปลี่ยน contract → อัปเดต `server/openapi.yaml` + `app/src/api/` พร้อมกัน
- อย่าคอมมิต: `data/content-planner.json`, `backups/`, `.env*`, `node_modules/`, `target/`, `dist/`
