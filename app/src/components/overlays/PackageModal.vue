<script setup lang="ts">
import { computed, ref } from 'vue'
import { choosePlan, currentUser } from '@/core/auth'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'selected', plan: string): void
}>()

const billingCycle = ref<'monthly' | 'yearly'>('monthly')
const busyPlan = ref<string | null>(null)
const successMsg = ref('')

const currentPlanId = computed(() => {
  const p = currentUser.value?.plan?.toLowerCase() ?? 'free'
  if (p === 'business') return 'agency'
  if (p === 'pro') return 'growth'
  return p
})

interface PackageTier {
  id: string
  name: string
  badge?: string
  popular?: boolean
  description: string
  monthlyPrice: string
  yearlyPrice: string
  unit: string
  features: string[]
}

const PACKAGES: PackageTier[] = [
  {
    id: 'starter',
    name: 'Starter',
    description: 'สำหรับครีเอเตอร์เดี่ยวและฟรีแลนซ์ เริ่มต้นวางแผนเป็นระบบ',
    monthlyPrice: '฿390',
    yearlyPrice: '฿312',
    unit: '/ เดือน',
    features: [
      '1 บัญชีผู้ใช้ / 1 Workspace',
      'ปฏิทินคอนเทนต์ 12 เดือน (Master Planner)',
      'Content Studio & Markdown Editor',
      'คลังไอเดีย Backlog & แฮชแท็กกรุ๊ป',
      'ระบบ Brand Identity & พาเลตต์สี',
    ],
  },
  {
    id: 'growth',
    name: 'Growth',
    description: 'สำหรับแบรนด์และทีมการตลาดขนาดเล็ก 2–5 คน',
    monthlyPrice: '฿1,290',
    yearlyPrice: '฿1,032',
    unit: '/ เดือน',
    features: [
      'สูงสุด 5 สมาชิกในทีม (Team RBAC)',
      'จัดการแคมเปญ & บันทึกบัญชีรายรับรายจ่าย',
      'Dashboard สถิติการเติบโตยอดวิว & Follower',
      'เชื่อมต่อ Multi-Platform Social OAuth',
      'อัปโหลด Custom Fonts & Brand Assets',
    ],
  },
  {
    id: 'agency',
    name: 'Agency Pro',
    badge: '★ The Killer Wedge',
    popular: true,
    description: 'สำหรับเอเจนซีและทีมโปรดักชันที่ต้องการปิดงานตรวจรวดเร็ว',
    monthlyPrice: '฿3,900',
    yearlyPrice: '฿3,120',
    unit: '/ เดือน',
    features: [
      'Guest Magic Link ส่งตรวจงานจากมือถือ ไม่ต้องล็อกอิน',
      'Video Timecode Proofing (ตรวจวิดีโอระดับวินาที)',
      'Visual Diff Slider (ตรวจเทียบภาพ Draft 1 vs 2)',
      'ส่งออก Executive White-Label PDF Report',
      'ไม่จำกัดจำนวนชิ้นงานและผู้ส่งตรวจ',
      'ระบบ Priority Video Transcode พรีวิวรวดเร็ว',
    ],
  },
  {
    id: 'enterprise',
    name: 'Enterprise',
    badge: 'Scale',
    description: 'สำหรับองค์กรและมีเดียเฮาส์ที่ดูแลลูกค้าหลายสิบแบรนด์',
    monthlyPrice: '฿8,900',
    yearlyPrice: '฿7,120',
    unit: '/ เดือน',
    features: [
      'ไม่จำกัดจำนวน Workspaces และแบรนด์ลูกค้า',
      'Custom Domain พอร์ทัลตรวจงาน (review.agency.com)',
      'Dedicated Object Storage & High Concurrency',
      'เข้าถึงเครือข่าย On-Demand Micro-Editing',
      'ทีมซัพพอร์ตและผู้จัดการดูแลระบบเฉพาะราย 24/7',
    ],
  },
]

async function selectPackage(planId: string): Promise<void> {
  if (busyPlan.value) return
  busyPlan.value = planId
  successMsg.value = ''
  try {
    await choosePlan(planId)
    successMsg.value = `เปลี่ยนแพ็กเกจเป็น ${planId.toUpperCase()} สำเร็จ`
    emit('selected', planId)
    setTimeout(() => {
      successMsg.value = ''
      emit('close')
    }, 900)
  } catch (err) {
    console.error('Failed to change plan:', err)
  } finally {
    busyPlan.value = null
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="pkg-modal-backdrop fixed inset-0 z-[1100] bg-black/60 backdrop-blur-md flex items-center justify-center p-4 sm:p-6 overflow-y-auto"
      @click.self="emit('close')"
    >
      <div
        class="pkg-modal-card relative w-full max-w-6xl max-h-[92vh] overflow-y-auto bg-surface text-ink border border-line rounded-2xl p-6 sm:p-8 shadow-2xl transition-all"
        role="dialog"
        aria-modal="true"
        aria-labelledby="pkg-title"
      >
        <!-- Close Button -->
        <button
          type="button"
          class="btn-close-modal absolute top-5 right-5 w-9 h-9 rounded-full bg-wash/80 hover:bg-wash flex items-center justify-center text-muted hover:text-ink transition-colors border 0"
          aria-label="Close"
          @click="emit('close')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>

        <!-- Header -->
        <div class="text-center max-w-2xl mx-auto mb-6">
          <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-wash text-ink/80 border border-line/60 mb-2">
            Workspace Packages &amp; Pricing
          </span>
          <h2 id="pkg-title" class="text-2xl sm:text-3xl font-display font-bold tracking-tight text-ink m-0">
            เลือกแพ็กเกจเพื่อยกระดับงานคอนเทนต์
          </h2>
          <p class="text-sm text-muted mt-2 leading-relaxed">
            เลือกโซลูชันที่ตอบโจทย์ขนาดทีมของคุณ ปลดล็อกระบบตรวจงานคลิปวิดีโอระดับวินาที และ Guest Magic Link
          </p>

          <!-- Billing Cycle Pill Toggle -->
          <div class="inline-flex items-center gap-1 p-1 bg-wash border border-line rounded-full mt-4">
            <button
              type="button"
              class="w-auto px-4 py-1.5 text-xs font-semibold rounded-full border-0 transition-all cursor-pointer"
              :class="billingCycle === 'monthly' ? 'bg-ink text-surface shadow-xs font-bold' : 'text-muted hover:text-ink'"
              @click="billingCycle = 'monthly'"
            >
              รายเดือน (Monthly)
            </button>
            <button
              type="button"
              class="w-auto px-4 py-1.5 text-xs font-semibold rounded-full border-0 transition-all cursor-pointer flex items-center gap-1.5"
              :class="billingCycle === 'yearly' ? 'bg-ink text-surface shadow-xs font-bold' : 'text-muted hover:text-ink'"
              @click="billingCycle = 'yearly'"
            >
              <span>รายปี (Yearly)</span>
              <span class="px-1.5 py-0.2 rounded-full text-[10px] bg-emerald-500/15 text-emerald-600 font-bold">
                ประหยัด 20%
              </span>
            </button>
          </div>
        </div>

        <!-- Success Toast -->
        <div
          v-if="successMsg"
          class="mb-4 p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-600 text-sm font-semibold text-center animate-fade-in"
        >
          ✓ {{ successMsg }}
        </div>

        <!-- Packages Grid (4 Columns) -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mt-6">
          <div
            v-for="pkg in PACKAGES"
            :key="pkg.id"
            class="pkg-card relative flex flex-col justify-between p-5 sm:p-6 rounded-2xl border transition-all duration-200"
            :class="[
              pkg.popular
                ? 'border-ink bg-wash/50 shadow-lg ring-1 ring-ink'
                : 'border-line bg-surface hover:border-line-hover shadow-xs',
              currentPlanId === pkg.id ? 'ring-2 ring-emerald-500/70 border-emerald-500' : ''
            ]"
          >
            <!-- Badge -->
            <div v-if="pkg.badge" class="absolute -top-3 left-1/2 -translate-x-1/2">
              <span
                class="px-2.5 py-0.5 rounded-full text-[11px] font-bold shadow-xs whitespace-nowrap"
                :class="pkg.popular ? 'bg-ink text-surface' : 'bg-wash text-ink border border-line'"
              >
                {{ pkg.badge }}
              </span>
            </div>

            <!-- Current Plan Tag -->
            <div v-if="currentPlanId === pkg.id" class="mb-2">
              <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-emerald-500/10 text-emerald-600 font-bold text-[11px]">
                ● แพ็กเกจปัจจุบันของคุณ
              </span>
            </div>

            <!-- Tier Head -->
            <div>
              <h3 class="text-xl font-bold font-display text-ink m-0">{{ pkg.name }}</h3>
              <p class="text-xs text-muted mt-1 min-h-[32px] leading-snug">{{ pkg.description }}</p>

              <!-- Price Display -->
              <div class="mt-4 pb-4 border-b border-line flex items-baseline gap-1.5">
                <span class="text-3xl font-extrabold tracking-tight text-ink font-display">
                  {{ billingCycle === 'yearly' ? pkg.yearlyPrice : pkg.monthlyPrice }}
                </span>
                <span class="text-xs text-muted font-medium">{{ pkg.unit }}</span>
              </div>

              <!-- Features List -->
              <ul class="mt-4 space-y-2.5 text-xs text-ink/85 list-none p-0">
                <li v-for="(feat, idx) in pkg.features" :key="idx" class="flex items-start gap-2">
                  <svg class="w-4 h-4 text-emerald-600 flex-none mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                  <span class="leading-tight">{{ feat }}</span>
                </li>
              </ul>
            </div>

            <!-- Action Button -->
            <div class="mt-6 pt-4 border-t border-line/60">
              <button
                type="button"
                class="w-full py-2.5 px-4 rounded-xl text-xs font-bold transition-all flex items-center justify-center gap-2 cursor-pointer border 0"
                :class="[
                  currentPlanId === pkg.id
                    ? 'bg-wash text-muted cursor-default border border-line'
                    : pkg.popular
                    ? 'btn-primary bg-ink text-surface hover:opacity-90 shadow-sm'
                    : 'bg-wash hover:bg-wash/80 text-ink border border-line hover:border-ink'
                ]"
                :disabled="currentPlanId === pkg.id || busyPlan === pkg.id"
                @click="selectPackage(pkg.id)"
              >
                <span v-if="busyPlan === pkg.id" class="inline-block animate-spin text-sm">⟳</span>
                <span v-if="currentPlanId === pkg.id">แพ็กเกจปัจจุบัน</span>
                <span v-else-if="pkg.popular">เลือก Agency Pro</span>
                <span v-else>เลือก {{ pkg.name }}</span>
              </button>
            </div>
          </div>
        </div>

        <!-- Footer Help / Note -->
        <div class="mt-8 pt-4 border-t border-line text-center text-xs text-muted flex flex-wrap items-center justify-between gap-3">
          <span>ต้องการแพ็กเกจสำหรับองค์กรแบบกำหนดเอง? ติดต่อทีมงานได้ที่ <a href="mailto:support@teenueng.com" class="text-ink font-semibold underline">support@teenueng.com</a></span>
          <span class="muted">ยกเลิกหรือเปลี่ยนแพ็กเกจได้ทุกเมื่อ · ปลอดภัย 100%</span>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.pkg-modal-backdrop {
  animation: fadeIn 0.2s ease-out;
}
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
