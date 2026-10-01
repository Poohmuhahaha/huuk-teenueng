// The router runs in hash mode, so a deep link typed without its fragment —
// `/members` instead of `/#/members` — would silently land on the first deck
// screen. Fold any real path into the fragment before the router mounts.
export function rewriteDeepLink(loc: {
  pathname: string
  search: string
  hash: string
}): string | null {
  const path = loc.pathname === '/index.html' ? '/' : loc.pathname
  if (path === '/' || loc.hash) return null
  return `/#${path}${loc.search}`
}

// A navbar date pick (YYYY-MM-DD) jumps to the Monthly Planner at that month.
export function monthOfDate(iso: string | null): number | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso ?? '')
  if (!m) return null
  const month = Number(m[2])
  if (month < 1 || month > 12) return null
  return month
}

export function plannerPathForDate(iso: string | null): string | null {
  const month = monthOfDate(iso)
  return month === null ? null : `/planner/${month}`
}
