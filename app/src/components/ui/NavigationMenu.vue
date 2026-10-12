<script setup lang="ts">
defineProps<{
  items: {
    id: string
    label: string
    href?: string
    children?: { id: string; label: string; href?: string; description?: string }[]
  }[]
  active?: string
}>()
const emit = defineEmits<{ select: [string] }>()
</script>

<template>
  <nav class="flex items-center gap-1">
    <div v-for="it in items" :key="it.id" class="group relative">
      <a
        v-if="it.href && !it.children"
        class="inline-flex items-center gap-1 w-auto bg-transparent border-0 font-sans text-[13px] font-semibold py-2 px-3 cursor-pointer no-underline transition-colors rounded-none hover:text-ink"
        :class="active === it.id ? 'text-ink underline underline-offset-4' : 'text-muted'"
        :href="it.href"
        @click="emit('select', it.id)"
      >
        {{ it.label }}
      </a>
      <button
        v-else
        type="button"
        class="inline-flex items-center gap-1 w-auto bg-transparent border-0 font-sans text-[13px] font-semibold py-2 px-3 cursor-pointer no-underline transition-colors rounded-none hover:text-ink"
        :class="active === it.id ? 'text-ink underline underline-offset-4' : 'text-muted'"
      >
        {{ it.label }} <span aria-hidden="true">▾</span>
      </button>
      <div
        v-if="it.children"
        class="absolute z-[60] top-full left-0 min-w-[240px] bg-surface border border-ink p-1.5 shadow-[4px_4px_0_var(--color-ink)] rounded-none hidden flex-col group-hover:flex group-focus-within:flex"
      >
        <a
          v-for="child in it.children"
          :key="child.id"
          class="flex flex-col gap-0.5 p-2 px-2.5 text-ink text-[13px] no-underline rounded-none hover:bg-wash"
          :href="child.href"
          @click="emit('select', child.id)"
        >
          <strong class="font-semibold">{{ child.label }}</strong>
          <span v-if="child.description" class="text-[11px] text-muted">{{ child.description }}</span>
        </a>
      </div>
    </div>
  </nav>
</template>
