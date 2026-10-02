# Huuk — Screen States (ทุก state ในทุกหน้า สำหรับ wireframe)

> ทุก **12 หน้า** ต้องออกแบบ state ให้ครบ ไม่ใช่แค่ตอนมีข้อมูล
> ที่มา: อ่านโค้ดจริง (`pages/*`, `components/*`) — loading/empty/error/no-perm/reconnect/conflict/locked ฯลฯ

## 1) มาตรฐาน state ที่ต้องมี (taxonomy)
| # | State | ความหมาย | UI มาตรฐาน |
|---|---|---|---|
| S0 | **Boot / gate** | แอปกำลัง restore session / ยังไม่พร้อม | boot loader · login gate · workspace gate |
| S1 | **Loading** | กำลังโหลด query (มี `isPending`) | skeleton/`Loading…` |
| S2 | **Success (data)** | มีข้อมูล | เนื้อหาเต็ม |
| S3 | **Empty** | โหลดเสร็จแต่ไม่มีข้อมูล | empty card + CTA |
| S4 | **Error** | query/mutation ล้มเหลว | error card + **Try again** |
| S5 | **No permission / read-only** | ไม่มีสิทธิ์ (FE `can()`) | disable/hide + tooltip `auth.noPerm` |
| S6 | **Busy / pending (action)** | กำลัง submit/save | ปุ่ม disabled + spinner/label |
| S7 | **Validation error** | 400 จากกติกา | inline alert ใต้ฟอร์ม |
| S8 | **Conflict** | 409 (version/lock/ซ้ำ) | banner/alert + ทางเลือก |
| S9 | **Reconnect needed** | token หมดอายุ / ยังไม่เชื่อม | ปุ่ม **Reconnect** |
| S10 | **Not found / 404** | slug/resource ไม่มี | not-found card |
| S11 | **Success notice** | action สำเร็จ | toast/banner ชั่วคราว |
| S12 | **First-run / onboarding** | ครั้งแรก | callout/onboarding hint |
| S13 | **Locked by other** | ถูกล็อกโดยคนอื่น | ปิดการแก้ + ชื่อผู้ล็อก |
| S14 | **Rate limited** | 429 | ข้อความ “ลองใหม่ใน N วินาที” |

---

## 2) Global gate (App.vue — ก่อนถึงทุกหน้า)
| State | Trigger | UI |
|---|---|---|
| S0 | mount → `restore()` | boot loader `Loading…` |
| gate-login | `authRequired && !isLoggedIn` | แสดง `LoginPage` |
| gate-workspace | ล็อกอินแล้วแต่ยังไม่มี workspace | แสดง `WorkspaceSetupPage` |
| S10 | route ไม่ match | `NotFoundPage` |

---

## 3) Home — `/dashboard`
| State | Trigger | UI |
|---|---|---|
| S1 | posts/metrics/live กำลังโหลด | `Computing…` / `Loading…` |
| S2 | มีข้อมูล | KPI 6 ใบ · Live grid · Active campaigns · charts · Top-5 |
| S3 | ไม่มีโพสต์/แคมเปญ | ค่า KPI = 0 · empty ใน Live/campaign |
| S4 | posts หรือ metrics error | error card + **Try again** |
| S9 | Live `hasToken=false` / `live.error` | ข้อความ + **Reconnect Meta** |
| S5 | Ads glance อ่านอย่างเดียว / Top-5 computed | glance read-only · **Edit** เปิด ProtectedModal |
| S13/S8 | กด Edit ที่ computed cell | `ProtectedModal` (Cancel/Confirm + “ไม่ต้องแสดง 5 นาที”) |

## 4) Plan — `/plan` (tabs: Monthly · Calendar · Ideas · Hashtags)
**Monthly**
| State | Trigger | UI |
|---|---|---|
| S1 | posts กำลังโหลด | `Loading rows…` |
| S2 | มีโพสต์ | master table + row editor |
| S3 | ไม่มีโพสต์เดือนนี้ | empty card + **+ New topic** |
| S4 | โหลดพลาด | error card + **Try again** |
| S6 | add/save/lock pending | ปุ่ม `Adding…`/`Saving…` disabled |
| S7/S8 | validate 400 · lock 409 | alert ใต้ editor |
| S13 | `lockedBy` เป็นคนอื่น | ปิด Save/Duplicate/Delete + “Locked by {name}” |
| S5 | ไม่มี `posts.write`/`posts.lock` | disable + tooltip |
| S2’ | เปิดจาก `?post=id` | เลือกแถวนั้นอัตโนมัติ |

**Calendar**
| State | Trigger | UI |
|---|---|---|
| S1 | posts/setup โหลด | `Loading…` |
| S2 | มีโพสต์ | CalendarGrid WEEK 1–6 |
| S3 | เดือนว่าง / filter ไม่เจอ | ตารางว่าง |

**Ideas**
| State | Trigger | UI |
|---|---|---|
| S1 | ideas โหลด | `Loading ideas…` |
| S2 | มีไอเดีย | table + Capture form |
| S3 | ไม่มีไอเดีย | ตารางว่าง |
| S6/S7 | toggle/promote/add pending · error | alert |
| S11 | promote สำเร็จ | notice |
| S5 | ไม่มี `ideas.write` | disable + tooltip |

**Hashtags**
| State | Trigger | UI |
|---|---|---|
| S1 | groups โหลด | `Loading groups…` |
| S2 | มีกลุ่ม | HashtagGroups |
| S3 | ไม่มีกลุ่ม | ว่าง |
| S4 | โหลดพลาด | error card + **Try again** |
| S5 | ไม่มี `hashtags.write` | fieldset disabled |

## 5) Content — `/content` (tabs: Studio · Feed preview)
**Studio — list**
| State | Trigger | UI |
|---|---|---|
| S1 | list โหลด | `Loading content…` |
| S2 | มีรายการ | list items |
| S3 | ว่าง / search ไม่เจอ / status ว่าง | `No content here yet.` |
| S4 | list error | error card + **Try again** |
| gate | ยังไม่ล็อกอิน | sign-in card |

**Studio — editor**
| State | Trigger | UI |
|---|---|---|
| S1 | content โหลด | `Loading content…` |
| S2 | โหลดเสร็จ | editor + preview |
| S6 | กำลัง saving/autosave | `Saving…` |
| S11 | saved | `Saved` · `Unsaved changes` |
| S8 | version 409 | conflict banner → **Reload latest / Keep mine** |
| S7 | title ว่าง/version ผิด | 400/409 alert |
| S5 | `content.write` ไม่มี | `canEdit=false` → disable ฟิลด์/ปุ่ม |
| status | draft/review/scheduled/published/archived | ปุ่มต่างกัน (Submit review / Back to draft / Unpublish / Restore to draft / Archive) |
| S2’ | published | share card + **Copy link / Open** |
| S3 | ยังไม่เลือกชิ้นงาน | `Select a piece` card |
| S12 | schedule วันที่อดีต | inline `Pick a future date…` |
| confirm | delete/restore | native confirm |

**Feed preview**
| State | Trigger | UI |
|---|---|---|
| S1 | posts/setup โหลด | `Loading…` |
| S2 | มีโพสต์ | FeedGrid 3×3 |
| S3 | ไม่มีโพสต์ | empty grid |
| S4 | โหลดพลาด | error card + **Try again** |

## 6) Promote — `/promote` (tabs: Campaigns · Meta Ads)
**Campaigns**
| State | Trigger | UI |
|---|---|---|
| S1 | campaigns โหลด | `Loading config…` |
| S2 | มีแคมเปญ | list + editor |
| S3 | ไม่มีแคมเปญ | `No campaigns yet.` |
| S2’ | ยังไม่เลือก | `Pick a campaign…` |
| S6/S11 | dirty/saving/saved | `Saved.` |
| S7 | save error | alert |
| S5 | `campaigns.write/delete` ไม่มี | disable + tooltip |
| confirm | delete | native confirm |
| S10 | เปิด `/campaigns/:id` ที่ไม่มี | ไม่เลือก (ว่าง) |

**Meta Ads**
| State | Trigger | UI |
|---|---|---|
| S1 | ads โหลด | `Loading config…` |
| S2 | มีแคมเปญ | account + KPI tiles + table + audit |
| S3 | ไม่มีแคมเปญ / ไม่มี ad account | `No campaigns mirrored yet — press Sync ads.` / `No ad accounts…` |
| S4 | load/sync/action error | alert |
| S9 | Meta ต้อง login ใหม่ | **Reconnect Meta** |
| S5 | management ปิด (`ads_manage=false`) | badge `Read-only` + ปุ่มจัดการ disabled (`ads.manageNeeded`) |
| S6 | sync/action pending | `Syncing…` disabled |
| S7 | budget ≤ 0 | `Enter a budget greater than zero.` |
| confirm | เปิด management | dialog ยืนยัน |
| S3’ | ไม่มี adset/ad | `No ad sets…` / `No ads…` |
| empty audit | ยังไม่มี action | ซ่อน section audit |

## 7) Analyze — `/analyze` (tabs: Performance · Finance)
**Performance**
| State | Trigger | UI |
|---|---|---|
| S1 | posts/setup โหลด | `Loading…` |
| S2 | มีข้อมูล | import card + goals table + chart + per-post table |
| S3 | ไม่มี metrics | ตารางว่าง / ค่า 0 |
| S4 | โหลดพลาด | error card + **Try again** |
| S6 | import pending | `Importing…` |
| S11/S7 | import สำเร็จ / ล้มเหลว | ข้อความผลลัพธ์ |
| S5 | ไม่มี `metrics.import` | **Import** disabled + tooltip |

**Finance**
| State | Trigger | UI |
|---|---|---|
| S1 | txns โหลด | `Loading ledger…` |
| S2 | มีรายการ | BudgetCards + charts + TxnTable |
| S3 | ไม่มีรายการ | ตารางว่าง |
| S6 | add pending | ปุ่ม loading |
| S7 | amount ผิด/kind ผิด | alert |
| S5 | ไม่มี `finance.write` | disable + tooltip |

## 8) Settings — `/settings` (tabs: Brand · Workspace · Members · Connections)
**Brand**
| State | Trigger | UI |
|---|---|---|
| S1 | brand โหลด | `Loading brand kit…` |
| S2 | มีข้อมูล | ครบทุก section |
| S4 | โหลดพลาด | error card + **Try again** |
| S5 | ไม่มี `brand.write` | disable + tooltip |
| S6 | upload/save pending | `Uploading…` / `Saving…` |
| S11/S7 | copied/imported/invalid | notice / alert |
| oversize | รูป > 500px | `Resize this image` |
| S12 | มาจาก onboarding | onboarding callout |

**Workspace (config + permissions + accounts)**
| State | Trigger | UI |
|---|---|---|
| S1 | setup โหลด | `Loading config…` |
| S2 | มีข้อมูล | fields + option lists + users + accounts + roles |
| S5 | ไม่มี `setup.write`/`users.manage` | read-only hint + disable |
| S6 | add/save pending | ปุ่ม loading |
| S7 | role/permission ผิด | 400 alert |
| S11 | temp password | กล่องโชว์ครั้งเดียว + **Copy** |
| confirm | remove user/account | tooltip/disable (ลบตัวเองไม่ได้) |

**Members**
| State | Trigger | UI |
|---|---|---|
| S1 | members โหลด | `Loading config…` |
| S2 | มีสมาชิก | list |
| S3 | ไม่มีสมาชิก | `No members yet…` |
| S4 | โหลด/แอ็กชัน error | alert |
| not-owner | ไม่ใช่เจ้าของ | `Only the workspace owner can manage members.` |
| S6/S11 | add pending / added / removed | notice |

**Connections**
| State | Trigger | UI |
|---|---|---|
| S1 | platforms โหลด | `Loading…` |
| per-status | disconnected / connected / error | status dot + connect/manage/reconnect |
| S6 | connect/sync pending | `Connecting…` / `Syncing…` |
| pick | หลาย Page | modal เลือก Page |
| S9 | `hasToken=false` | `Reconnect` |
| S5 | ไม่มี `platforms.manage` | disable + tooltip |

## 9) Welcome — `/welcome`
S1 loading config · S2 idle (form) · S6 submitting (`Loading…`) · S7 name ว่าง (ปุ่ม disabled) · S4 error · S11 สำเร็จ → เข้าแอป

## 10) Login — `/login`
S2 idle · S6 submitting · S14 429 (`ลองใหม่ใน N วินาที`) · S4 401 (ข้อความกลาง ๆ) · link ไป Register (ซ่อนถ้าปิดรับสมัคร) · สำเร็จ → redirect

## 11) Register — `/register`
registration-closed (`Registration is currently closed.` + ไม่มีฟอร์ม) · S2 idle · S6 submitting · S7 password < 12 · S8 409 (อีเมล/ชื่อซ้ำ) · S4 error · S11 สำเร็จ → `/plans`

## 12) Plans — `/plans`
S2 idle (3 การ์ด + badge Most popular + Current package) · S6 busy (`Loading…`) · logged-out click → `/register` · S4 error

## 13) Read — `/read`, `/read/:slug`
**Index:** S1 loading · S2 list · S3 `Nothing published yet.`
**Article:** S1 loading · S2 article (meta/title/excerpt/hero/body/tags) · S10 not found (`Post not found` + hint + All posts)

## 14) 404 — `*`
Static: title · message · **Back to the first screen** · **Open the planner**

---

## 15) Matrix สรุป (หน้าหลัก × state)
| Page | S1 Load | S2 Data | S3 Empty | S4 Error | S5 No-perm | S6 Busy | S7 Valid | S8 Conflict | S9 Reconnect | S10 404 | S11 Notice | S12 Onboard | S13 Locked |
|---|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| Home | ✓ | ✓ | ✓ | ✓ | ✓ | | | | ✓ | | | | ✓ |
| Plan | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | | | | ✓ |
| Content | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | | ✓ | | |
| Promote | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | ✓ | ✓ | ✓ | | |
| Analyze | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | | | ✓ | | |
| Settings | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | ✓ | | ✓ | ✓ | |
| Welcome | ✓ | ✓ | | ✓ | | ✓ | ✓ | | | | ✓ | ✓ | |
| Login | | ✓ | | ✓ | | ✓ | | | | | | | |
| Register | | ✓ | | ✓ | | ✓ | ✓ | ✓ | | | ✓ | | |
| Plans | | ✓ | | ✓ | | ✓ | | | | | | | |
| Read | ✓ | ✓ | ✓ | ✓ | | | | | | ✓ | | | |
| 404 | | | | | | | | | | ✓ | | | |

## 16) Overlays — state ที่ต้องมี
| Overlay | Loading | Error | Busy | Success | Empty/Other |
|---|:--:|:--:|:--:|:--:|---|
| Account menu | – | – | – | – | logged-out (login/register) |
| Settings sheet | ✓ | ✓ | ✓ | ✓ | read-only hint |
| LoginModal | – | ✓ | ✓ | – | mode login/register |
| ChangePasswordModal | – | ✓ | ✓ | ✓ | mismatch confirm |
| ProtectedModal | – | – | – | – | checkbox “ไม่ต้องแสดง 5 นาที” |
| PlatformLogin | ✓ | ✓ | ✓ | ✓ | mock vs redirect · pick vs connected |
| OAuth picker | ✓ | ✓ | ✓ | – | empty pages |
| Guide | – | – | – | – | – |
| DatePickerPopup | – | – | – | – | clear/today |
| Ads dialogs | – | ✓ | ✓ | – | validation |
| Publish menu | – | ✓ | ✓ | – | schedule future check |
| Revisions drawer | ✓ | ✓ | ✓ | ✓ | no revisions |
