<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useData } from 'vitepress'
import { loadMonaco, monacoThemeFor } from './monacoSetup'
import type { editor } from 'monaco-editor'

const props = withDefaults(
  defineProps<{
    modelValue: string
    language: 'matlab' | 'toml'
    minHeight?: string
  }>(),
  { minHeight: '360px' },
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'change'): void
}>()

const { isDark } = useData()

const container = ref<HTMLDivElement | null>(null)
let instance: editor.IStandaloneCodeEditor | null = null
let applyingExternal = false

onMounted(async () => {
  const monaco = await loadMonaco()
  if (!container.value) return
  // Monaco measures glyph widths from `fontFamily` itself, so it needs the
  // resolved font list rather than a CSS variable reference.
  const monoFont =
    getComputedStyle(container.value).getPropertyValue('--vp-font-family-mono').trim() ||
    'monospace'
  instance = monaco.editor.create(container.value, {
    value: props.modelValue,
    language: props.language,
    theme: monacoThemeFor(isDark.value),
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    fontSize: 13,
    fontFamily: monoFont,
    tabSize: 2,
    padding: { top: 12 },
    automaticLayout: true,
    scrollbar: { verticalScrollbarSize: 10, horizontalScrollbarSize: 10 },
  })
  instance.onDidChangeModelContent(() => {
    if (applyingExternal || !instance) return
    emit('update:modelValue', instance.getValue())
    emit('change')
  })
  watch(isDark, (dark) => {
    monaco.editor.setTheme(monacoThemeFor(dark))
  })
})

watch(
  () => props.modelValue,
  (value) => {
    if (instance && value !== instance.getValue()) {
      applyingExternal = true
      instance.setValue(value)
      applyingExternal = false
    }
  },
)

onBeforeUnmount(() => {
  instance?.dispose()
  instance = null
})
</script>

<template>
  <div ref="container" class="pg-monaco" :style="{ minHeight }"></div>
</template>
