# requirements.md — EXAMPLE: Landing Page (`/landing`)

> ตัวอย่าง spec ครบที่ implement จริงแล้วใน repo นี้
> (`app/src/pages/LandingPage.vue` + route + `landing.test.ts`)
> Workflow: Requirements-First (product-led, ไม่มี tech constraints)

- Feature: public marketing landing at `/landing`
- Intent: หน้าแนะนำ Huuk + พาคนเข้าแอป (CTAs ไป `/register`, `/login`, `/plans`, `/`)
  ภาษาอังกฤษล้วน, static (ไม่มี queries), sections: Hero + 6 hubs + Pricing + Demo

## User stories

### Visitors discover Huuk and enter the app

- As a **visitor**, I want to see what Huuk does in one page so that I decide to try it
- Acceptance:
  - [ ] hero อธิบาย product + มีปุ่ม Start free / Open workspace / See packages
  - [ ] 6 hubs แสดงพร้อม tabs + access notes + ลิงก์เปิด hub
  - [ ] pricing 3 tiers (Free/Pro/Business) ตรงกับ `/plans`
  - [ ] demo accounts 3 ตัว + password + ปุ่ม login

### Logged-in users are not pushed to re-onboard

- As a **signed-in user**, I want the header to show "Back to workspace"
  so that I am not pushed to login/register again
- Acceptance:
  - [ ] `loggedIn` → header มีลิงก์ `/` แทน `/login`

## Requirements (EARS)

```
REQ-L01  WHEN an anonymous user visits /landing
         THE SYSTEM SHALL render the public page without the AppShell (meta.public)
REQ-L02  WHEN the landing renders
         THE SYSTEM SHALL show hero (h1 + sub + 3 CTAs) linking to
         /register, / and /plans respectively
REQ-L03  WHEN the landing renders
         THE SYSTEM SHALL list all six hubs (Home/Plan/Content/Promote/Analyze/Settings)
         each with its tab names, access note and an Open-hub link
REQ-L04  WHEN the landing renders
         THE SYSTEM SHALL show pricing tiers Free/Pro/Business with the same
         plans.* copy as /plans (Free → Continue with Free → /register;
         paid → Contact sales link to https://teenueng.com)
REQ-L05  WHEN the landing renders
         THE SYSTEM SHALL show demo accounts owner/editor/client (@studio.local)
         with role labels and password demo1234 plus a Log in button each
REQ-L06  WHEN a signed-in user visits /landing
         THE SYSTEM SHALL show "Back to workspace" (to /) instead of "Log in"
REQ-L07  WHEN the landing renders
         THE SYSTEM SHALL use English-only copy from core/i18n landing.* keys
         (no hardcoded marketing strings outside i18n)
REQ-L08  WHEN the landing is tested
         THE SYSTEM SHALL be covered by landing.test.ts asserting by
         role/text/tag (h1, buttons, h3 hubs, tiers, demo emails, link hrefs)
```

## Non-goals

- ไม่ทำไทย/สลับภาษา (English only ตามมติ)
- ไม่ย้าย `/` (deck redirect เดิมอยู่ครบ — `/landing` แยกใหม่)
- ไม่แตะ backend/openapi (static page, ไม่มี endpoint ใหม่ → 4-place rule ไม่เกี่ยว)

## Review

- [x] EARS วัดได้ + มี ID — [x] public/standalone ถูก (meta.public)
- [x] ไม่แตะ auth/permissions/locks/CMS → Quick-able แต่ทำ standard ก็ได้ (งานนี้เล็ก)
