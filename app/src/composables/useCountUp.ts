import { ref, watch, onUnmounted, type Ref } from 'vue'

export interface CountUpOptions {
  duration?: number // in ms, default 950ms
  delay?: number // in ms, default 0ms
}

function parseFormattedNumber(input: string | number) {
  if (typeof input === 'number') {
    return {
      prefix: '',
      val: input,
      decimals: Number.isInteger(input) ? 0 : 1,
      suffix: '',
    }
  }

  const str = String(input).trim()
  const m = str.match(/^(.*?)(\d[\d,]*(?:\.\d+)?)(.*?)$/)
  if (!m) return null

  const [, prefix, numStr, suffix] = m
  const cleanNum = numStr.replace(/,/g, '')
  const val = parseFloat(cleanNum)
  const decimals = numStr.includes('.') ? numStr.split('.')[1].length : 0

  return { prefix, val, decimals, suffix }
}

function easeOutQuart(x: number): number {
  return 1 - Math.pow(1 - x, 4)
}

export function useCountUp(
  target: Ref<string | number> | (() => string | number),
  options: CountUpOptions = {},
) {
  const { duration = 950, delay = 0 } = options
  const display = ref('')
  let rafId: number | null = null
  let prevVal = 0
  let isFirstRun = true

  function formatValue(current: number, prefix: string, decimals: number, suffix: string) {
    const formatted = current.toLocaleString('en-US', {
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals,
    })
    return `${prefix}${formatted}${suffix}`
  }

  function startAnimation() {
    const raw = typeof target === 'function' ? target() : target.value

    if (
      typeof window === 'undefined' ||
      (typeof process !== 'undefined' && process.env?.NODE_ENV === 'test')
    ) {
      display.value = String(raw)
      return
    }

    if (rafId) {
      cancelAnimationFrame(rafId)
      rafId = null
    }

    const parsed = parseFormattedNumber(raw)

    if (!parsed || Number.isNaN(parsed.val)) {
      display.value = String(raw)
      return
    }

    // Respect prefers-reduced-motion
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      display.value = String(raw)
      return
    }

    const { prefix, val: endVal, decimals, suffix } = parsed
    // On first mount, start from 0; on subsequent prop changes, animate from previous value
    const startVal = isFirstRun ? 0 : prevVal
    isFirstRun = false
    prevVal = endVal

    let startTime: number | null = null

    function step(timestamp: number) {
      if (!startTime) startTime = timestamp
      const elapsed = timestamp - startTime

      if (elapsed < delay) {
        rafId = requestAnimationFrame(step)
        return
      }

      const progress = Math.min((elapsed - delay) / duration, 1)
      const eased = easeOutQuart(progress)
      const current = startVal + (endVal - startVal) * eased

      display.value = formatValue(current, prefix, decimals, suffix)

      if (progress < 1) {
        rafId = requestAnimationFrame(step)
      } else {
        // Guarantee exact target string on completion
        display.value = String(raw)
        rafId = null
      }
    }

    rafId = requestAnimationFrame(step)
  }

  watch(
    () => (typeof target === 'function' ? target() : target.value),
    () => {
      startAnimation()
    },
    { immediate: true },
  )

  onUnmounted(() => {
    if (rafId) {
      cancelAnimationFrame(rafId)
      rafId = null
    }
  })

  return display
}
