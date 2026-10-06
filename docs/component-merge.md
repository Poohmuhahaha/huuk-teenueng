# Component Merge — Huuk ← Pugu

**Policy:** Huuk is the primary component set (Vue 3). Pugu components are never discarded:
anything Huuk lacked was ported into Huuk so both projects share one library.

## Ported into Huuk (11)

| Pugu source | Huuk target | Notes |
|---|---|---|
| `ui/Alert.svelte` | `components/ui/Alert.vue` | added 4 tones (info/success/warning/danger) |
| `ui/Avatar.svelte` | `components/ui/Avatar.vue` | restyled square + hairline (Huuk design) |
| `ui/AvatarGroup.svelte` | `components/ui/AvatarGroup.vue` | `items`, `max`, +N overflow |
| `ui/CodeBlock.svelte` | `components/ui/CodeBlock.vue` | `code`, `lang`, default slot |
| `ui/EmptyState.svelte` | `components/ui/EmptyState.vue` | `title`, `message`, action slot |
| `ui/ProgressBar.svelte` | `components/ui/ProgressBar.vue` | clamped 0–100, ARIA |
| `ui/Skeleton.svelte` | `components/ui/Skeleton.vue` | `line` / `block`, `width` |
| `ui/StatCards.svelte` | `components/ui/StatCards.vue` | serif values per Huuk type |
| `ui/StatusDot.svelte` | `components/ui/StatusDot.vue` | `on`, `title` |
| `Pagination.svelte` | `components/ui/Pagination.vue` | `currentPage` / `totalPages`, `update:currentPage` |
| `ConfirmModal.svelte` | `components/overlays/ConfirmModal.vue` | `open`, `title`, `message`, `confirmText`, `tone`; emits `cancel` / `confirm` |

## Already in Huuk (no port needed)

`BarChart` · `Chip` · `KpiCard` · `StatusChip` · `StatusFunnel`

## Deliberately not ported

| Pugu source | Reason |
|---|---|
| `WebWorkspaceSwitcher.svelte` | app-specific API (`/api/v1/me`, `/workspaces/:id/switch`); Huuk has its own workspace switcher in `AppShell` |
| `Editor.svelte` | superseded by Huuk `ContentEditor` |
| `admin/ConfirmModal.svelte`, `admin/Pagination.svelte` | duplicates of the ported versions |

Pugu's original `.svelte` files remain untouched in `PRODUCTIVITY-prod/Pugu`.

## Where the merged components live

- Exports: `app/src/components/ui/index.ts`, `app/src/components/overlays/index.ts`
- Live gallery: `/#/components` (families + component index with file paths)
- Verified: `bun run typecheck`, `bun run test` (207 passed), `bun run build`

## Later expansion — shadcn-parity (2026-10-02)

Added 39 components to Huuk so coverage matches shadcn/ui scale (no deps, monochrome design):

- **Forms**: Input, Textarea, Label, Field, InputGroup, Checkbox, RadioGroup, Switch, Slider, PinInput (OTP), Select, Combobox
- **Display**: Badge, Separator, Kbd, Spinner, AspectRatio, Breadcrumb, ListItem, ButtonGroup, ScrollArea
- **Navigation**: Tabs, Accordion, Collapsible, Toggle, ToggleGroup, NavigationMenu, Menubar, Sidebar
- **Menus & overlays**: Popover, Tooltip, HoverCard, DropdownMenu, ContextMenu, CommandPalette, Dialog, Drawer, Toast, Resizable

Shared helper: `src/composables/useDismiss.ts` (click-outside + Escape).
Gallery index is now **81 components** at `/#/components`.
