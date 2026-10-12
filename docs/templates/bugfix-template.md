# bugfix.md — TEMPLATE

> วิธีเขียนดู `docs/08-bugfix-specs.md` — `bugfix.md` แทน `requirements.md`

- Bug: `<1 ประโยค>`
- Severity: `<critical path? เคย regression? root ชัดไหม>`

## Reproduction steps (เป๊ะๆ)

1. `<state/mode>`
2. `<action>`
3. `<observed wrong result>`

## Current vs expected

- Current: `<defect>`
- Expected: `<correct behavior + status/message>`
- Constraints (ห้ามเปลี่ยน): `<…>`

## Unchanged behavior (กัน regression)

```
WHEN <condition that must keep working>
THEN the system SHALL CONTINUE TO <existing behavior>
```

## Root cause (เติมหลังวิเคราะห์)

- `<…>`

## Fix approach + tests

- `<…>` + property tests ครอบทั้ง fix และ preservation
- ถ้าสโคปบวมเป็นฟีเจอร์ใหม่ → เปิด Feature Spec แยก (ดู docs/08 §08.3)
