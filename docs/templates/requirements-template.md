# requirements.md — TEMPLATE

> copy ไฟล์นี้ไป `docs/examples/<feature>/requirements.md` แล้วเติม
> วิธีเขียนดู `docs/04-ears.md`, checklist ดู `docs/10-glossary-checklists.md`

- Feature: `<name>`
- Workflow: Requirements-First | Design-First (+ High/Low level)
- Intent (1–2 ประโยค): `<what + why>`

## User stories

### <Story 1: role + goal + benefit>

- As a `<role>`, I want `<goal>` so that `<benefit>`
- Acceptance:
  - [ ] `<testable criterion 1>`
  - [ ] `<testable criterion 2>`

## Requirements (EARS)

```
REQ-001  WHEN <condition/event>
         THE SYSTEM SHALL <expected behavior + status/message>
```

- Happy path: `REQ-001…`
- Edge cases: `REQ-…`
- Errors (400/401/403/404/409 + message ตรงตัว): `REQ-…`
- Modes (demo vs production ต่างกันตรงไหน): `REQ-…`

## Non-goals (ไม่ทำในสโคปนี้)

- `<explicitly out of scope>`

## Review (gate ก่อนไป design — ดู docs/10 §10.3)

- [ ] EARS ทุกข้อวัดได้ + มี ID + ไม่มี 2 SHALL ในข้อเดียว
- [ ] 401/403/404/409 ครบ paths ที่เกี่ยว + demo/production แยกชัด
- [ ] ผ่าน Analyze checklist (docs/07 §07.3)
