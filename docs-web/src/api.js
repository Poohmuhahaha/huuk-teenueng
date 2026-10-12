// Strapi REST client (public find/findOne — no token needed).
export const STRAPI_URL =
  import.meta.env.VITE_STRAPI_URL || 'http://localhost:1337'

async function get(path) {
  const res = await fetch(`${STRAPI_URL}${path}`)
  if (!res.ok) throw new Error(`Strapi ${res.status}: ${path}`)
  const json = await res.json()
  return json.data ?? []
}

// Sidebar list: light fields only, ordered.
export function listPages() {
  const q = new URLSearchParams({
    'fields[0]': 'title',
    'fields[1]': 'slug',
    'fields[2]': 'section',
    'fields[3]': 'sectionLabel',
    'fields[4]': 'order',
    'fields[5]': 'summary',
    sort: 'order:asc',
    'pagination[pageSize]': '100',
  })
  return get(`/api/doc-pages?${q}`)
}

export async function getPage(slug) {
  const q = new URLSearchParams({
    'filters[slug][$eq]': slug,
    'fields[0]': 'title',
    'fields[1]': 'slug',
    'fields[2]': 'section',
    'fields[3]': 'order',
    'fields[4]': 'body',
  })
  const rows = await get(`/api/doc-pages?${q}`)
  return rows[0] ?? null
}
