// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest'
import { markdownToText, renderMarkdown } from '@/core/markdown'

describe('renderMarkdown', () => {
  it('renders headings, emphasis, lists, quotes and code', () => {
    const html = renderMarkdown('# Title\n\nHello **world** and *italics*.\n\n- one\n- two\n\n> quote\n\n`code`')
    expect(html).toContain('<h1>Title</h1>')
    expect(html).toContain('<strong>world</strong>')
    expect(html).toContain('<em>italics</em>')
    expect(html).toContain('<ul>')
    expect(html).toContain('<blockquote>')
    expect(html).toContain('<code>code</code>')
  })

  it('escapes raw HTML instead of executing it', () => {
    const html = renderMarkdown('<script>alert(1)</script>\n\n<img src=x onerror=alert(1)>')
    expect(html).not.toContain('<script>')
    expect(html).not.toContain('<img')
    expect(html).toContain('&lt;script&gt;')
    expect(html).toContain('&lt;img')
  })

  it('rejects javascript: links but keeps the label', () => {
    const html = renderMarkdown('[click](javascript:alert(1))')
    expect(html).not.toContain('javascript:')
    expect(html).toContain('click')
  })

  it('allows https links with rel=noopener and in-app hash links', () => {
    const html = renderMarkdown('[site](https://example.com) and [planner](#/planner/2)')
    expect(html).toContain('href="https://example.com"')
    expect(html).toContain('rel="noopener noreferrer"')
    expect(html).toContain('href="#/planner/2"')
  })

  it('keeps formatting markers inside code spans literal', () => {
    const html = renderMarkdown('`**not bold**`')
    expect(html).toContain('<code>**not bold**</code>')
    expect(html).not.toContain('<strong>')
  })

  it('markdownToText strips markup for summaries', () => {
    expect(markdownToText('# Hi\n\nThis is **bold** [x](https://y.z)', 40)).toBe('Hi This is bold x')
  })
})
