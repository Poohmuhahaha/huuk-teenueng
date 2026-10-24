<script setup lang="ts">
import { t } from '@/core/i18n'
import type { Brand } from '@/mock/db'

function capitalize(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1)
}

defineProps<{
  brand: Brand
  activeBrandBg: string
  customBgStyle: Record<string, string>
  logoSlots: Array<{ index: number; url: string }>
  paletteDraft: string[]
  styleDraft: { radius: number; fillOpacity: number; strokeWidth: number; shadow: string }
  moodboard: string[]
  logoLabels: Record<number, string>
  paletteSlots: readonly string[]
  paletteRoles: Record<string, string>
  paletteUsage: Record<string, string>
  readableOn: (hex: string) => string
  hexToRgb: (hex: string) => string
  brandInitials: (name: string) => string
}>()
</script>

<template>
  <div id="brand-guideline-pdf" class="brand-pdf-deck" :style="customBgStyle" aria-hidden="true">
    <!-- PAGE 1: 1. BRAND IDENTITY -->
    <section class="pdf-slide pdf-slide-hero" :style="{ backgroundImage: `url('${activeBrandBg}')` }">
      <div class="pdf-cover-shade"></div>
      <div class="pdf-cover-container">
        <header class="pdf-slide-header">
          <span class="pdf-slide-title">1. Brand Identity</span>
          <span class="pdf-header-channel">{{ brand.channel }}</span>
        </header>

        <div class="pdf-hero-content">
          <div class="pdf-hero-title">{{ brand.channel || 'Studio Channel' }}</div>
          <p v-if="brand.positioning" class="pdf-hero-pos">{{ brand.positioning }}</p>
        </div>

        <footer class="pdf-slide-footer">
          <span>{{ brand.channel }}</span>
          <span>1 / 7</span>
        </footer>
      </div>
    </section>

    <!-- PAGE 2: 2. BRAND VOICE -->
    <section class="pdf-slide pdf-slide-dark">
      <header class="pdf-slide-header">
        <span class="pdf-slide-title">2. Brand Voice</span>
        <span class="pdf-header-channel">{{ brand.channel }}</span>
      </header>

      <div class="pdf-slide-body">
        <div class="pdf-voice-meta">
          <div class="pdf-voice-field" v-if="brand.positioning">
            <span class="pdf-field-lbl">Positioning</span>
            <p class="pdf-field-val">{{ brand.positioning }}</p>
          </div>
          <div class="pdf-voice-field" v-if="brand.audience">
            <span class="pdf-field-lbl">Audience</span>
            <p class="pdf-field-val">{{ brand.audience }}</p>
          </div>
          <div class="pdf-voice-field" v-if="brand.voice">
            <span class="pdf-field-lbl">Voice &amp; Tone</span>
            <p class="pdf-field-val">{{ brand.voice }}</p>
          </div>
        </div>

        <div class="pdf-voice-grid">
          <div class="pdf-voice-card pdf-card-do">
            <div class="pdf-card-title">Do</div>
            <ul class="pdf-clean-list">
              <li v-for="d in brand.dos" :key="d">{{ d }}</li>
              <li v-if="!brand.dos.length" class="muted">Show the actual craft and process</li>
            </ul>
          </div>
          <div class="pdf-voice-card pdf-card-dont">
            <div class="pdf-card-title">Don't</div>
            <ul class="pdf-clean-list pdf-list-dont">
              <li v-for="d in brand.donts" :key="d">{{ d }}</li>
              <li v-if="!brand.donts.length" class="muted">Clickbait and artificial urgency</li>
            </ul>
          </div>
        </div>
      </div>

      <footer class="pdf-slide-footer">
        <span>{{ brand.channel }}</span>
        <span>2 / 7</span>
      </footer>
    </section>

    <!-- PAGE 3: 3. LOGOS -->
    <section class="pdf-slide pdf-slide-dark">
      <header class="pdf-slide-header">
        <span class="pdf-slide-title">3. Logos</span>
        <span class="pdf-header-channel">{{ brand.channel }}</span>
      </header>

      <div class="pdf-slide-body">
        <div class="pdf-logos-grid">
          <div v-for="slot in logoSlots" :key="slot.index" class="pdf-logo-card">
            <div
              class="pdf-logo-display"
              :class="{ 'on-photo': slot.index === 2 }"
              :style="slot.index === 2 ? { backgroundImage: `url('${activeBrandBg}')` } : {}"
            >
              <img v-if="slot.url" :src="slot.url" :alt="t(logoLabels[slot.index])" class="pdf-logo-img" />
              <div v-else class="pdf-logo-placeholder">
                <span class="pdf-logo-fallback">{{ brandInitials(brand.channel) }}</span>
              </div>
            </div>
            <div class="pdf-logo-info">
              <strong>{{ t(logoLabels[slot.index]) }}</strong>
              <span class="muted">
                {{ slot.index === 0 ? 'Primary logo' : slot.index === 1 ? 'Secondary lockup' : 'Social & icon stamp' }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <footer class="pdf-slide-footer">
        <span>{{ brand.channel }}</span>
        <span>3 / 7</span>
      </footer>
    </section>

    <!-- PAGE 4: 4. PALETTE -->
    <section class="pdf-slide pdf-slide-dark">
      <header class="pdf-slide-header">
        <span class="pdf-slide-title">4. Palette</span>
        <span class="pdf-header-channel">{{ brand.channel }}</span>
      </header>

      <div class="pdf-slide-body">
        <div class="pdf-palette-strip">
          <div
            v-for="slot in paletteSlots"
            :key="`strip-${slot}`"
            class="pdf-palette-strip-seg"
            :style="{
              background: paletteDraft[Number(slot) - 1] || '#111111',
              flex: slot === '1' ? '5' : slot === '2' ? '3' : slot === '3' ? '1.5' : '1',
            }"
          >
            <span :style="{ color: readableOn(paletteDraft[Number(slot) - 1] || '#111111') }">
              {{ t(paletteRoles[slot]) }}
            </span>
          </div>
        </div>

        <div class="pdf-palette-grid">
          <div v-for="slot in paletteSlots" :key="slot" class="pdf-palette-card">
            <div class="pdf-palette-swatch" :style="{ background: paletteDraft[Number(slot) - 1] || '#111111' }">
              <span class="pdf-palette-sample" :style="{ color: readableOn(paletteDraft[Number(slot) - 1] || '#111111') }">
                Aa
              </span>
            </div>
            <div class="pdf-palette-details">
              <strong>{{ t(paletteRoles[slot]) }}</strong>
              <span class="pdf-hex-code">{{ paletteDraft[Number(slot) - 1] || '#111111' }}</span>
              <span class="pdf-rgb-code">{{ hexToRgb(paletteDraft[Number(slot) - 1] || '#111111') }}</span>
              <p class="muted">{{ t(paletteUsage[slot]) }}</p>
            </div>
          </div>
        </div>
      </div>

      <footer class="pdf-slide-footer">
        <span>{{ brand.channel }}</span>
        <span>4 / 7</span>
      </footer>
    </section>

    <!-- PAGE 5: 5. TYPOGRAPHY -->
    <section class="pdf-slide pdf-slide-dark">
      <header class="pdf-slide-header">
        <span class="pdf-slide-title">5. Typography</span>
        <span class="pdf-header-channel">{{ brand.channel }}</span>
      </header>

      <div class="pdf-slide-body" :style="{ fontFamily: `'${brand.fonts[0] || 'Noto Sans Thai'}', sans-serif` }">
        <div class="pdf-type-lead">
          <div class="pdf-font-name">{{ brand.fonts[0] || 'Noto Sans Thai' }}</div>
          <span class="muted">Primary Typeface</span>
        </div>

        <div class="pdf-type-specimen-box">
          <div class="pdf-type-big-aa">Aa</div>
          <div class="pdf-type-charset">
            <div>ABCDEFGHIJKLMNOPQRSTUVWXYZ · abcdefghijklmnopqrstuvwxyz</div>
            <div>กขคฆงจฉชซฌญฎฏฐฑฒณดตถทธนบปผฝพฟภมยรลวศษสหฬอฮ 0123456789</div>
          </div>
        </div>

        <div class="pdf-type-preview-row">
          <div class="pdf-type-scale-item">
            <span class="pdf-scale-label">Display</span>
            <div class="pdf-scale-display">{{ brand.channel || 'Studio Channel' }}</div>
          </div>
          <div class="pdf-type-scale-item">
            <span class="pdf-scale-label">Body Text</span>
            <p class="pdf-scale-body">เนื้อหาที่จริงใจ ไม่เร่งรีบ ให้ความสำคัญกับกระบวนการและคุณค่าที่แท้จริง</p>
          </div>
        </div>
      </div>

      <footer class="pdf-slide-footer">
        <span>{{ brand.channel }}</span>
        <span>5 / 7</span>
      </footer>
    </section>

    <!-- PAGE 6: 6. STYLE -->
    <section class="pdf-slide pdf-slide-dark">
      <header class="pdf-slide-header">
        <span class="pdf-slide-title">6. Style</span>
        <span class="pdf-header-channel">{{ brand.channel }}</span>
      </header>

      <div class="pdf-slide-body">
        <div class="pdf-style-tokens-grid">
          <div class="pdf-style-token-card">
            <div class="pdf-token-text">
              <span class="pdf-token-lbl">Corner Radius</span>
              <strong>{{ styleDraft.radius }}px</strong>
              <p class="muted">Softened corners for cards, buttons, and media.</p>
            </div>
            <div class="pdf-token-preview">
              <div class="pdf-preview-box" :style="{ borderRadius: `${styleDraft.radius}px` }"></div>
            </div>
          </div>

          <div class="pdf-style-token-card">
            <div class="pdf-token-text">
              <span class="pdf-token-lbl">Surface Shadow</span>
              <strong>{{ capitalize(styleDraft.shadow) }}</strong>
              <p class="muted">Ambient elevation for overlays and dropdowns.</p>
            </div>
            <div class="pdf-token-preview">
              <div class="pdf-preview-shadow" :class="`shadow-${styleDraft.shadow}`"></div>
            </div>
          </div>

          <div class="pdf-style-token-card">
            <div class="pdf-token-text">
              <span class="pdf-token-lbl">Fill Opacity</span>
              <strong>{{ styleDraft.fillOpacity }}%</strong>
              <p class="muted">Surface density for solid layers.</p>
            </div>
            <div class="pdf-token-preview">
              <div class="pdf-preview-fill" :style="{ opacity: styleDraft.fillOpacity / 100 }"></div>
            </div>
          </div>
        </div>
      </div>

      <footer class="pdf-slide-footer">
        <span>{{ brand.channel }}</span>
        <span>6 / 7</span>
      </footer>
    </section>

    <!-- PAGE 7: 7. MOODBOARD -->
    <section class="pdf-slide pdf-slide-dark">
      <header class="pdf-slide-header">
        <span class="pdf-slide-title">7. Moodboard</span>
        <span class="pdf-header-channel">{{ brand.channel }}</span>
      </header>

      <div class="pdf-slide-body">
        <div v-if="moodboard.length > 0" class="pdf-mood-grid">
          <div
            v-for="(url, i) in moodboard"
            :key="`${url}-${i}`"
            class="pdf-mood-item"
            :style="{ backgroundImage: `url('${url}')` }"
          ></div>
        </div>
        <div v-else class="pdf-mood-hero-single" :style="{ backgroundImage: `url('${activeBrandBg}')` }">
          <div class="pdf-mood-single-shade"></div>
          <div class="pdf-mood-single-caption">
            <strong>Visual References</strong>
            <span class="muted">Brand photography and atmosphere anchor</span>
          </div>
        </div>
      </div>

      <footer class="pdf-slide-footer">
        <span>{{ brand.channel }}</span>
        <span>7 / 7</span>
      </footer>
    </section>
  </div>
</template>
