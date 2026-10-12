# tasks.md — TEMPLATE

> วิธีแตก + waves ดู `docs/06-tasks-doc.md` — ทำงานใน wave เดียวกันสลับได้ ห้ามข้าม wave

## Wave 1 (no dependencies — ขนานได้)

- [ ] T-01 `<backend handler + openapi.yaml>` (REQ-`<…>`)
      files: `server/src/handlers.rs`, `server/openapi.yaml`
      verify: `cargo test --manifest-path server/Cargo.toml <filter>`
- [ ] T-02 `<mock/api.ts function>` (REQ-`<…>`)
      files: `app/src/mock/api.ts`
- [ ] T-03 `<page skeleton + i18n keys>` (REQ-`<…>`)
      files: `app/src/pages/<…>.vue`, `app/src/core/i18n.ts`

## Wave 2 (รอ Wave 1)

- [ ] T-04 `<http.ts client>` (REQ-`<…>`) — ต้องตรงกับ T-01+T-02
      files: `app/src/api/http.ts`
- [ ] T-05 `<queries.ts hooks + invalidateQueries>` (REQ-`<…>`)
      files: `app/src/core/queries.ts`

## Wave 3 (รอ Wave 2)

- [ ] T-06 `<integration tests 2 ฝั่ง>` (REQ-`<…>`)
      files: `server/tests/api.rs`, `app/src/tests/<…>.test.ts`
      verify: `bun run build` + `bun run test`

## Optional (required ไม่เสร็จไม่ถือว่าปิดงาน)

- [ ] (optional) T-07 `<nice-to-have>`

## 4-place checklist (ทุก task ที่แตะ endpoint)

- [ ] handlers.rs + http.ts + mock/api.ts + openapi.yaml + tests 2 ฝั่ง
