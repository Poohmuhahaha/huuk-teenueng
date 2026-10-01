# Component — content-planner-huuk

| ชิ้นส่วน | ไฟล์ | หน้าที่ |
|---|---|---|
| Router | `app/src/app/router/index.ts` | hash-history, routes จาก `core/screens.ts` + `/login /register /plans /welcome /read/:slug` |
| API switch | `app/src/api/index.ts` | `VITE_API_URL` ว่าง→`mock/api.ts`, มีค่า→`api/http.ts` (contract เดียวกัน) |
| Shell | `app/src/components/layout/AppShell.vue`, `ScreensDeck.vue` | chrome + deck navigation/animation |
| Planner grid | `app/src/components/tables/MasterTable.vue` | ตารางงานหลัก |
| Editor | `app/src/components/editor/ContentEditor.vue` | Markdown + live preview + autosave + revisions |
| Auth UI | `app/src/components/overlays/PlatformLogin.vue`, `LoginModal.vue`, `SettingsPanel.vue` | login โซเชียลเต็มจอ + onboard client (Settings สร้าง account + รหัสชั่วคราว) |
| State/data | `app/src/core/{queries,auth,session,oauth}.ts` | TanStack queries, token/session |
| Backend core | `server/src/{lib,handlers,store,main}.rs` | Axum router/middleware, permission gates, seeded Store + snapshot ทุก 5s |
| Desktop | `app/src-tauri/` + `app/scripts/desktop-sidecar.sh` | Tauri sidecar |
