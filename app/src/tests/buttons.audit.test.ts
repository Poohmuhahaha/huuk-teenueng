// @vitest-environment happy-dom
/// <reference types="node" />
// Runtime button audit: mounts the real app, walks every screen, and clicks
// every button it can find while capturing thrown errors and console noise.
// Writes a machine-readable report to /tmp/opencode/buttons-runtime.json.
// Opt-in (skipped in normal runs):
//   BUTTON_AUDIT=1 npx vitest run src/components/__tests__/buttons.audit.test.ts
import { afterEach, beforeEach, describe, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { writeFileSync } from 'node:fs'
import App from '@/app/App.vue'
import router from '@/app/router'
import * as api from '@/mock/api'

const ROUTES = [
  '/brand',
  '/planner/2',
  '/calendar',
  '/feed',
  '/dashboard',
  '/performance',
  '/ideas',
  '/hashtags',
  '/finance',
  '/studio',
  '/read',
  '/nope',
] as const

interface ClickRecord {
  route: string
  index: number
  name: string
  source: 'text' | 'aria-label' | 'title' | 'none'
  disabled: boolean
  changed: boolean
  thrown: string | null
  console: string[]
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms))
}

async function settle(ms = 40): Promise<void> {
  await flushPromises()
  await sleep(ms)
  await flushPromises()
}

function buttonName(b: ReturnType<ReturnType<typeof mount>['findAll']>[number]): { name: string; source: ClickRecord['source'] } {
  const text = b.text().replace(/\s+/g, ' ').trim()
  if (text) return { name: text, source: 'text' }
  const aria = b.attributes('aria-label')
  if (aria) return { name: aria, source: 'aria-label' }
  const title = b.attributes('title')
  if (title) return { name: title, source: 'title' }
  return { name: '(unnamed)', source: 'none' }
}

describe.runIf(process.env.BUTTON_AUDIT === '1')('button audit', () => {
  let wrapper: ReturnType<typeof mount> | undefined
  let logs: string[] = []
  let errors: string[] = []
  let errorSpy: ReturnType<typeof vi.spyOn>
  let warnSpy: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    logs = []
    errors = []
    errorSpy = vi.spyOn(console, 'error').mockImplementation((...args: unknown[]) => {
      errors.push(args.map(String).join(' '))
    })
    warnSpy = vi.spyOn(console, 'warn').mockImplementation((...args: unknown[]) => {
      logs.push(args.map(String).join(' '))
    })
    vi.stubGlobal('confirm', () => true)
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText: async () => undefined },
    })
  })

  afterEach(() => {
    wrapper?.unmount()
    wrapper = undefined
    errorSpy.mockRestore()
    warnSpy.mockRestore()
    vi.unstubAllGlobals()
  })

  it('clicks every button on every route', async () => {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: 0, staleTime: 0 }, mutations: { retry: 0 } },
    })
    wrapper = mount(App, {
      global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
    })
    await router.isReady()
    await settle(120)

    const records: ClickRecord[] = []
    const extraRoutes: string[] = []
    try {
      const rows = await api.listContent()
      if (rows[0]) extraRoutes.push(`/studio/${rows[0].id}`)
      const publicRows = await api.listPublicContent()
      if (publicRows[0]) extraRoutes.push(`/read/${publicRows[0].slug}`)
    } catch {
      // best effort — audit without detail routes if the mock refuses
    }

    for (const route of [...ROUTES, ...extraRoutes]) {
      await router.push(route)
      await settle(160)
      const before = records.length
      for (let i = 0; i < 60; i++) {
        const buttons = wrapper.findAll('button')
        if (i >= buttons.length) break
        const b = buttons[i]
        const { name, source } = buttonName(b)
        const disabled = b.attributes('disabled') !== undefined
        const errBefore = errors.length
        let changed = false
        let thrown: string | null = null
        if (!disabled) {
          const htmlBefore = wrapper.html()
          try {
            await b.trigger('click')
            await settle(45)
            changed = wrapper.html() !== htmlBefore
          } catch (e) {
            thrown = e instanceof Error ? e.message : String(e)
          }
        }
        records.push({
          route,
          index: i,
          name,
          source,
          disabled,
          changed,
          thrown,
          console: errors.slice(errBefore).map((s) => s.slice(0, 300)),
        })
      }
      logs.push(`route ${route}: ${records.length - before} buttons examined`)
    }

    writeFileSync('/tmp/opencode/buttons-runtime.json', JSON.stringify(records, null, 2))
    const summary = {
      routes: ROUTES.length + extraRoutes.length,
      clicks: records.length,
      disabled: records.filter((r) => r.disabled).length,
      unnamed: records.filter((r) => r.source === 'none').length,
      thrown: records.filter((r) => r.thrown).length,
      consoleErrors: records.filter((r) => r.console.length > 0).length,
      silent: records.filter((r) => !r.disabled && !r.changed && !r.thrown && r.console.length === 0).length,
      warnings: logs.filter((l) => !l.startsWith('route ')).length,
    }
    console.info(`AUDIT SUMMARY ${JSON.stringify(summary)}`)
    const problems = records.filter((r) => r.thrown || r.console.length > 0)
    for (const p of problems) {
      console.info(`AUDIT PROBLEM ${p.route} [${p.index}] "${p.name}" thrown=${p.thrown} console=${p.console.join(' | ')}`)
    }
    for (const w of logs.filter((l) => !l.startsWith('route '))) {
      console.info(`AUDIT WARN ${w.slice(0, 300)}`)
    }
  }, 180_000)
})
