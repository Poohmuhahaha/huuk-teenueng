<script setup lang="ts">
const model = defineModel<string>()
withDefaults(
  defineProps<{ options?: { value: string; label: string; disabled?: boolean }[]; name?: string; direction?: 'row' | 'column' }>(),
  { options: () => [], name: 'p-radio', direction: 'column' },
)
</script>

<template>
  <div
    class="flex gap-2.5"
    :class="direction === 'row' ? 'flex-row flex-wrap gap-4' : 'flex-col'"
    role="radiogroup"
  >
    <label
      v-for="o in options"
      :key="o.value"
      class="group inline-flex items-center gap-2 cursor-pointer text-sm text-ink select-none"
      :class="{ 'opacity-50 cursor-not-allowed': o.disabled }"
    >
      <input
        type="radio"
        class="peer sr-only"
        :name="name"
        :value="o.value"
        :checked="model === o.value"
        :disabled="o.disabled"
        @change="model = o.value"
      />
      <span
        class="inline-flex items-center justify-center w-[18px] h-[18px] shrink-0 border border-line rounded-full bg-surface transition-colors peer-checked:border-ink peer-focus-visible:outline-2 peer-focus-visible:outline-ink peer-focus-visible:outline-offset-2"
        aria-hidden="true"
      >
        <span
          class="w-2 h-2 rounded-full transition-colors"
          :class="model === o.value ? 'bg-ink' : 'bg-transparent'"
        />
      </span>
      <span>{{ o.label }}</span>
    </label>
  </div>
</template>
