// Tiny, dependency-free Markdown renderer for the content studio.
//
// Safety model: raw text is HTML-escaped first, then a fixed set of tags is
// inserted around it. Only `http(s)`, root-relative and in-app hash links are
// allowed, so user content can never inject markup or javascript: URLs.
function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function safeUrl(url: string): string | null {
  const trimmed = url.trim()
  if (/^https?:\/\//i.test(trimmed) || trimmed.startsWith('/') || trimmed.startsWith('#/')) {
    return trimmed
  }
  return null
}

/** Inline formatting. `text` must already be HTML-escaped. */
function renderInline(escaped: string): string {
  let out = escaped
  const codeSpans: string[] = []

  // Code spans are extracted first so nothing else formats their contents.
  out = out.replace(/`([^`]+)`/g, (_match, code: string) => {
    codeSpans.push(code)
    return `\u0000${codeSpans.length - 1}\u0000`
  })

  out = out.replace(/!\[([^\]]*)\]\(([^)\s]+)\)/g, (_match, alt: string, url: string) => {
    const safe = safeUrl(url)
    return safe ? `<img src="${safe}" alt="${alt}" loading="lazy" />` : _match
  })

  out = out.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_match, label: string, url: string) => {
    const safe = safeUrl(url)
    if (!safe) return label
    const external = /^https?:\/\//i.test(safe)
    const rel = external ? ' target="_blank" rel="noopener noreferrer"' : ''
    return `<a href="${safe}"${rel}>${label}</a>`
  })

  out = out.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
  out = out.replace(/__([^_]+)__/g, '<strong>$1</strong>')
  out = out.replace(/(^|[^*])\*([^*\n]+)\*/g, '$1<em>$2</em>')
  out = out.replace(/(^|[^_])_([^_\n]+)_/g, '$1<em>$2</em>')

  return out.replace(/\u0000(\d+)\u0000/g, (_match, index: string) => `<code>${codeSpans[Number(index)]}</code>`)
}

/** Block-level renderer returning an HTML string built only from escaped text. */
export function renderMarkdown(markdown: string): string {
  const lines = (markdown ?? '').replace(/\r\n?/g, '\n').split('\n')
  const html: string[] = []
  let paragraph: string[] = []
  let list: 'ul' | 'ol' | null = null
  let quote: string[] = []
  let inCode = false
  let codeLines: string[] = []

  const flushParagraph = (): void => {
    if (paragraph.length) {
      html.push(`<p>${paragraph.map((line) => renderInline(escapeHtml(line))).join('<br />')}</p>`)
      paragraph = []
    }
  }
  const flushList = (): void => {
    if (list) {
      html.push(`</${list}>`)
      list = null
    }
  }
  const flushQuote = (): void => {
    if (quote.length) {
      html.push(`<blockquote>${quote.map((line) => `<p>${renderInline(escapeHtml(line))}</p>`).join('')}</blockquote>`)
      quote = []
    }
  }
  const flushAll = (): void => {
    flushParagraph()
    flushList()
    flushQuote()
  }

  for (const raw of lines) {
    if (inCode) {
      if (raw.trim().startsWith('```')) {
        html.push(`<pre><code>${escapeHtml(codeLines.join('\n'))}</code></pre>`)
        inCode = false
        codeLines = []
      } else {
        codeLines.push(raw)
      }
      continue
    }
    if (raw.trim().startsWith('```')) {
      flushAll()
      inCode = true
      codeLines = []
      continue
    }
    if (!raw.trim()) {
      flushAll()
      continue
    }

    const heading = /^(#{1,6})\s+(.*)$/.exec(raw)
    if (heading) {
      flushAll()
      const level = heading[1].length
      html.push(`<h${level}>${renderInline(escapeHtml(heading[2]))}</h${level}>`)
      continue
    }
    if (/^\s*(---|\*\*\*|___)\s*$/.test(raw)) {
      flushAll()
      html.push('<hr />')
      continue
    }
    const quoteMatch = /^>\s?(.*)$/.exec(raw)
    if (quoteMatch) {
      flushParagraph()
      flushList()
      quote.push(quoteMatch[1])
      continue
    }
    const unordered = /^\s*[-*+]\s+(.*)$/.exec(raw)
    if (unordered) {
      flushParagraph()
      flushQuote()
      if (list !== 'ul') {
        flushList()
        html.push('<ul>')
        list = 'ul'
      }
      html.push(`<li>${renderInline(escapeHtml(unordered[1]))}</li>`)
      continue
    }
    const ordered = /^\s*\d+[.)]\s+(.*)$/.exec(raw)
    if (ordered) {
      flushParagraph()
      flushQuote()
      if (list !== 'ol') {
        flushList()
        html.push('<ol>')
        list = 'ol'
      }
      html.push(`<li>${renderInline(escapeHtml(ordered[1]))}</li>`)
      continue
    }
    flushList()
    flushQuote()
    paragraph.push(raw.trim())
  }

  if (inCode) {
    html.push(`<pre><code>${escapeHtml(codeLines.join('\n'))}</code></pre>`)
  }
  flushAll()
  return html.join('\n')
}

/** Plain-text preview (for cards/summaries), derived from the markdown. */
export function markdownToText(markdown: string, max = 160): string {
  const text = (markdown ?? '')
    .replace(/```[\s\S]*?```/g, ' ')
    .replace(/!\[[^\]]*\]\([^)]*\)/g, ' ')
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/[#>*_`~-]+/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
  return text.length > max ? `${text.slice(0, max - 1)}…` : text
}
