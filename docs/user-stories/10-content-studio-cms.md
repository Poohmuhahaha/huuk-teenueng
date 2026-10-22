# E10 — Content Studio (CMS)

**โมดูล:** `handlers.rs` (content CRUD/publish/schedule/revisions/public)
**Storage:** `Store.content: Vec<Content>` — global; `Version` (optimistic concurrency), `revisions` (cap 20)
**FE:** `pages/StudioPage.vue`, `components/editor/ContentEditor.vue`, `core/markdown.ts`
**Permission:** `content.write` (สร้าง/แก้/duplicate/restore), `content.publish` (publish/unpublish/schedule), `content.delete` (ลบ)

---

## แนวคิด
- เนื้อหา 3 ชนิด: `article` / `page` / `note`; สถานะ 5: `draft/review/scheduled/published/archived`
- **Optimistic concurrency:** ทุก mutation ส่ง `version` ที่อ่านมา; ไม่ตรง → **409**
- ทุกการแก้ push revision (snapshot ก่อนแก้) cap 20
- `promote_due_scheduled` ทำงานแบบ lazy ตอนมี content read (ไม่มี background timer)

## User Stories

| ID | As a | I want to | So that | Priority |
|---|---|---|---|---|
| US-10-01 | Editor/Client | ดูรายการ content + ค้นหา/กรอง status | หาเนื้อหา | Must |
| US-10-02 | Editor/Client | สร้าง content ใหม่ | เริ่มเขียน | Must |
| US-10-03 | Editor/Client | เขียน Markdown + preview | ดูตัวอย่าง | Must |
| US-10-04 | Editor/Client | autosave ทุก ~1.5 วิ | ไม่หลุดงาน | Must |
| US-10-05 | Editor/Client | แก้ชนกันแล้วเลือก reload/keep mine | ไม่ทับงานคนอื่น | Must |
| US-10-06 | Editor/Client | publish | เผยแพร่ | Must |
| US-10-07 | Editor/Client | unpublish | ถอน | Should |
| US-10-08 | Editor/Client | ตั้งเวลา publish | วางแผน | Must |
| US-10-09 | Editor/Client | duplicate | ต่อจากของเดิม | Should |
| US-10-10 | Owner | ลบ content | เอาเนื้อหาออก | Should |
| US-10-11 | Editor | ดูประวัติ revision + preview | ย้อนดู | Should |
| US-10-12 | Editor | restore revision | ย้อนกลับ | Should |
| US-10-13 | Editor | คัดลอกลิงก์ public | แชร์ | Could |

---

## US-10-01 — List
`GET /api/content?status=&q=` (read) → `ContentSummary[]`
- list เรียก `promote_due_scheduled` (write lock) ก่อน; กรอง status และ q (title/excerpt/tags); เรียง `updatedAt` desc
- FE: ค้นหา debounce 250ms; แท็บ status; loading/error/empty

## US-10-02 — Create
`POST /api/content` · `content.write`
- บังคับ `id=c-...`, `author`, `version=1`, ล้าง publish/schedule/revisions
- status อื่นนอกจาก draft/review → **coerce เป็น draft**
- slug ว่าง → `slugify(title)` + `unique_slug`; validate capacity

## US-10-03/04/05 — Edit, Autosave, Conflict
**Get:** `GET /api/content/{id}` (read) → `Content` (revision.body ถูกตัดเป็น "")
**Update:** `PATCH /api/content/{id}` · `content.write` · body มี `version` + ฟิลด์ + `note?`

**Autosave (FE):**
1. `dirty = JSON.stringify(draft) !== baseline`
2. `watch(draft, deep)` debounce **1.5s** → `save(undefined, true)` (หยุดเมื่อ dirty=false / !canEdit / conflict / saving / !canWrite)
3. `save()` ส่งฟิลด์ editable ทั้งหมด + `version: loadedVersion` + note
4. Server: `check_content_version` (ไม่มี version → 400; ไม่ตรง → 409 พร้อม version ปัจจุบัน)
5. `push_revision` (snapshot ก่อนแก้) → `unique_slug` → `validate_content` → `version+1`, `updatedAt`
6. สำเร็จ → FE re-seed baseline + version
7. **409** → FE ขึ้น conflict banner: "Reload latest" (`reloadLatest`) หรือ "Keep mine" (`keepMine` = set loadedVersion = serverVersion + save override note)

**Status ผ่าน patch:** จำกัดเฉพาะ draft/review/archived (publish/schedule มี endpoint แยก)

**Acceptance**
- [ ] autosave เงียบทุก ~1.5s ไม่รบกวนการพิมพ์
- [ ] version ไม่ตรง → 409 + แบนเนอร์เลือกได้
- [ ] "Keep mine" ทับด้วยเวอร์ชันที่ตั้งใจ

## US-10-06/07 — Publish / Unpublish
**Publish:** `POST /api/content/{id}/publish {version}` · `content.publish`
- version ต้องตรง; title ต้องไม่ว่าง (400) → status=published + publishedAt + ล้าง schedule + revision "Published"
**Unpublish:** `POST /api/content/{id}/unpublish {version}` → status=draft + ล้าง schedule + revision "Unpublished" (**ไม่ล้าง publishedAt**)

## US-10-08 — Schedule
`POST /api/content/{id}/schedule {version, scheduledFor}` · `content.publish`
- `scheduledFor` ต้อง RFC3339 และเป็นอนาคต (ไม่งั้น 400) → status=scheduled + scheduledFor + revision "Scheduled"
- เมื่อถึงเวลา → `promote_due_scheduled` (lazy) เปลี่ยนเป็น published

## US-10-09/10 — Duplicate / Delete
- Duplicate: `POST /api/content/{id}/duplicate` · `content.write` → copy, title+" (copy)", unique slug, draft, version 1, ล้าง publish/schedule/revisions
- Delete: `DELETE /api/content/{id}` · `content.delete` → retain; 404 ถ้าไม่มี (**ไม่มี version guard**)

## US-10-11/12 — Revisions
- List: `GET /api/content/{id}/revisions` (read) → มี body ครบ, ใหม่สุดก่อน
- Get: `GET /api/content/{id}/revisions/{revision}` (read) — **FE ยังไม่มี hook นี้ (unused)**
- Restore: `POST /api/content/{id}/revisions/{revision}/restore {version}` · `content.write`
  - version ต้องตรง; revision ต้องมี (404)
  - `push_revision` (snapshot ปัจจุบัน) → คืน title/body/excerpt/seo/tags และ status **เฉพาะ draft/review ไม่งั้นบังคับ draft**; ไม่คืน kind/slug/heroImageUrl; ล้าง schedule; version++

## US-10-13 — Public link
FE สร้าง `${origin}${pathname}#/read/{slug}` → copy/open

---

## State — Content lifecycle

```mermaid
stateDiagram-v2
  [*] --> draft: create_content (coerce non-draft/review → draft)
  draft --> review: update_content status=review
  review --> draft: update_content status=draft
  draft --> archived: update_content status=archived
  review --> published: publish_content (title required, version checked)
  published --> draft: unpublish_content (publishedAt คงอยู่)
  draft --> scheduled: schedule_content (RFC3339 อนาคต)
  review --> scheduled: schedule_content
  scheduled --> published: promote_due_scheduled (lazy, ตอนมี read)
  published --> published: publish_content ซ้ำ (version++)
  archived --> draft: update_content status=draft
  draft --> draft: restore_content_revision (บังคับ draft)
```

## Sequence — Autosave, conflict & publish

```mermaid
sequenceDiagram
  autonumber
  actor E as Editor/Client
  participant CE as ContentEditor
  participant Q as TanStack Query
  participant API as handlers.rs
  participant ST as Store

  CE->>Q: useContentItem(id) → GET /api/content/:id
  API->>ST: promote_due_scheduled (write lock)
  API-->>CE: Content (revision.body = "")

  loop dirty ทุก ~1.5s
    CE->>Q: useUpdateContent({id, patch, version})
    Q->>API: PATCH /api/content/:id
    API->>ST: check_content_version
    alt ไม่ส่ง version
      API-->>Q: 400
    else version ไม่ตรง
      API-->>Q: 409 + version ปัจจุบัน
      Q-->>CE: conflict banner
      E->>CE: เลือก Reload latest หรือ Keep mine
      opt Keep mine
        CE->>CE: loadedVersion = serverVersion
        CE->>Q: PATCH ซ้ำ (override note)
      end
    else ตรง
      ST->>ST: push_revision + unique_slug + validate + version+1
      API-->>Q: 200 Content
      Q->>Q: re-seed baseline + loadedVersion
    end
  end

  E->>CE: กด Publish
  CE->>CE: ensureSaved()
  CE->>Q: usePublishContent({id, version})
  Q->>API: POST /api/content/:id/publish {version}
  API->>ST: check_content_version + title ไม่ว่าง?
  alt title ว่าง/version ไม่ตรง
    API-->>Q: 400/409
  else ok
    ST->>ST: status=published + publishedAt + revision "Published"
    API-->>Q: 200 Content
  end
```

## Sequence — Schedule & lazy promote / restore revision

```mermaid
sequenceDiagram
  autonumber
  actor E as Editor
  participant CE as ContentEditor
  participant API as handlers.rs
  participant ST as Store

  E->>CE: ตั้งวันเวลาในอนาคต → Schedule
  CE->>API: POST /api/content/:id/schedule {version, scheduledFor}
  API->>ST: parse RFC3339 + future check + version check
  alt ผิด
    API-->>CE: 400/409
  else ok
    ST->>ST: status=scheduled + scheduledFor + revision "Scheduled"
    API-->>CE: 200 Content
  end

  Note over ST: เมื่อถึงเวลา — ไม่มี background timer
  E->>API: GET /api/content (read ปกติ)
  API->>ST: promote_due_scheduled
  ST->>ST: scheduled && scheduledFor <= now → published + publishedAt + version++
  API-->>E: list ที่อัปเดตแล้ว

  E->>CE: เลือก revision → Restore
  CE->>API: POST /api/content/:id/revisions/:rev/restore {version}
  API->>ST: หา revision + version check
  alt ไม่พบ/ไม่ตรง
    API-->>CE: 404/409
  else ok
    ST->>ST: push_revision (ปัจจุบัน) + คืน title/body/seo/tags, status forced draft
    API-->>CE: 200 Content
  end
```

## Flow — Studio overview

```mermaid
flowchart TD
  Studio["/studio/:id?"] --> List["GET /api/content?status&q"]
  List --> New{"action"}
  New -->|"สร้าง"| C["POST /api/content (content.write)"]
  C --> CU["router → /studio/:id"]
  New -->|"เลือก"| Open["GET /api/content/:id"]
  Open --> Edit["แก้ + autosave PATCH (version)"]
  Edit --> V{"409?"}
  V -->|yes| Conflict["Reload latest / Keep mine"]
  V -->|no| Saved["version++ + revision"]
  Open --> Pub["POST /publish (content.publish)"]
  Open --> Unp["POST /unpublish"]
  Open --> Sch["POST /schedule"]
  Open --> Dup["POST /duplicate (content.write)"]
  Open --> Del["DELETE (content.delete)"]
  Open --> Rev["GET /revisions + POST /restore"]
  Pub --> PubOk["status=published"]
  Sch --> Prom["promote_due_scheduled (lazy)"]
  Prom --> PubOk
```
