<script setup lang="ts">
// SaaS packages. Free continues straight into the workspace; paid tiers are
// handled by the main site until billing is wired up.
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { choosePlan, currentUser } from '@/core/auth'
import { t } from '@/core/i18n'

const router = useRouter()
const busy = ref('')
const error = ref('')

const loggedIn = computed(() => currentUser.value !== null)
const currentPlan = computed(() => currentUser.value?.plan ?? null)

interface Plan {
  id: string
  name: string
  tag: string
  price: string
  features: string[]
  popular?: boolean
}

const plans: Plan[] = [
  { id: 'free', name: 'Free', tag: 'Community Edition สำหรับใช้งานเดี่ยว', price: 'ฟรีตลอดชีพ',
    features: ['1 บัญชีผู้ใช้ / 1 Workspace', 'ปฏิทินคอนเทนต์ 12 เดือน', 'Content Studio & Markdown Editor', 'ระบบ Brand Identity'] },
  { id: 'starter', name: 'Starter', tag: 'สำหรับครีเอเตอร์เดี่ยว & ฟรีแลนซ์', price: '฿390 / เดือน',
    features: ['ปฏิทินคอนเทนต์ไม่จำกัด', 'Content Studio & Export', 'คลังไอเดีย Backlog & แท็ก', 'Cloud Auto Sync'] },
  { id: 'growth', name: 'Growth', tag: 'สำหรับแบรนด์และทีม 2–5 คน', price: '฿1,290 / เดือน',
    features: ['สูงสุด 5 ที่นั่งในทีม (RBAC)', 'จัดการแคมเปญ & บันทึกค่าใช้จ่าย', 'Dashboard สถิติยอดวิว & ผู้ติดตาม', 'เชื่อมต่อ Social OAuth'] },
  { id: 'agency', name: 'Agency Pro', tag: 'The Killer Wedge สำหรับเอเจนซี', price: '฿3,900 / เดือน', popular: true,
    features: ['Guest Magic Link ตรวจงานมือถือ', 'Video Timecode Proofing ระดับวินาที', 'Visual Diff Slider ตรวจเทียบภาพ', 'ส่งออก White-Label PDF Report'] },
  { id: 'enterprise', name: 'Enterprise', tag: 'สำหรับองค์กร & มีเดียเฮาส์', price: '฿8,900 / เดือน',
    features: ['ไม่จำกัด Workspaces & แบรนด์', 'Custom Domain ตรวจงาน', 'Dedicated S3 Storage & Priority Transcode', 'ทีมซัพพอร์ต 24/7'] },
]

async function continueFree(): Promise<void> {
  if (!loggedIn.value) {
    await router.push('/register')
    return
  }
  error.value = ''
  busy.value = 'free'
  try {
    await choosePlan('free')
    await router.push('/brand?onboarding=1')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = ''
  }
}

async function selectTier(planId: string): Promise<void> {
  if (!loggedIn.value) {
    await router.push('/register')
    return
  }
  error.value = ''
  busy.value = planId
  try {
    await choosePlan(planId)
    await router.push('/brand?onboarding=1')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = ''
  }
}
</script>

<template>
  <div class="planspage">
    <div class="plans-card">
      <div class="plans-head">
        <strong>Huuk</strong><span class="lp-by">by teenueng</span>
        <span class="spacer" />
        <RouterLink v-if="loggedIn" class="btn" to="/">{{ t('plans.backToApp') }}</RouterLink>
        <RouterLink v-else class="btn" to="/login">{{ t('auth.login') }}</RouterLink>
      </div>
      <h1>{{ t('plans.title') }}</h1>
      <p class="plans-sub">{{ t('plans.subtitle') }}</p>
      <div class="plans-grid">
        <div v-for="plan in plans" :key="plan.id" class="plan-card" :class="{ popular: plan.popular }">
          <span v-if="plan.popular" class="plan-badge">Popular</span>
          <strong>{{ plan.name }}</strong>
          <p class="plan-tag">{{ plan.tag }}</p>
          <p class="plan-price">{{ plan.price }}</p>
          <ul class="plan-list">
            <li v-for="f in plan.features" :key="f">{{ f }}</li>
          </ul>
          <div class="plan-cta">
            <button v-if="plan.id === 'free'" class="btn btn-primary w-full"
              :disabled="busy === 'free'" @click="continueFree">
              {{ busy === 'free' ? t('common.loading') : t('plans.continueFree') }}
            </button>
            <button v-else class="btn w-full" :class="plan.popular ? 'btn-primary' : ''"
              :disabled="busy === plan.id || currentPlan === plan.id" @click="selectTier(plan.id)">
              {{ busy === plan.id ? t('common.loading') : currentPlan === plan.id ? 'แพ็กเกจปัจจุบัน' : `เลือก ${plan.name}` }}
            </button>
          </div>
          <p v-if="currentPlan === plan.id" class="plan-current">{{ t('plans.current') }}</p>
        </div>
      </div>
      <p v-if="error" class="autherr" role="alert">{{ error }}</p>
      <div class="plans-foot">
        <span class="muted">teenueng.com · huuk.teenueng.com</span>
      </div>
    </div>
  </div>
</template>
