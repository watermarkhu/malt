<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { withBase } from 'vitepress'
import MonacoEditor from './MonacoEditor.vue'

interface Warning {
  rule_id: string
  severity: string
  message: string
  line: number
  column: number
  fix?: unknown
}

// The WebAssembly package is built from the same commit and embedded in the site
// (`mise run build-wasm && mise run serve-local-wasm`, done by the Docs workflow).
const WASM_URL = withBase('/assets/malt-wasm/malt_lib.js')

const EXAMPLES: Record<string, string> = {
  common: `x = 1
for i = 1:10
    results(i) = i * 2
end
disp(results)`,
  prealloc: `% Without preallocation, the array grows inside the loop.
results = [];
for i = 1:1000
    results = [results, i^2];
end

% Preallocate instead: AGROW won't fire.
results = zeros(1, 1000);
for i = 1:1000
    results(i) = i^2;
end`,
  class: `classdef MyClass < handle
    properties
        Value
    end
    methods
        function set.Value(obj, val)
            obj.Value = val;
        end
        function obj = MyClass()
            obj.Value = 0;
        end
    end
end`,
  clean: `function output = compute(input)
    % A well-formed function: semicolons, preallocation, and clear names.
    arguments
        input double
    end
    output = zeros(1, numel(input));
    for idx = 1:numel(input)
        output(idx) = input(idx) * 2;
    end
end`,
}

const loading = ref(true)
const loadError = ref('')
const version = ref('')
const input = ref('')
const warnings = ref<Warning[]>([])
const exampleKey = ref('')

const DEFAULT_TOML = `# malt playground configuration (.malt.toml).
# Everything here is optional; comments-only uses malt's defaults.
# Off by default (uncomment a block to re-enable):
#   code-generation, deployment, system-objects,
#   configuration-issues, suggested-improvements, unsupported,
#   and naming + custom-checks (via the default "mathworks" preset).
# preset = "all" under [lint] turns everything on.
#
# [lint.categories]
# code-generation = "error"
#
# [lint.rules.CODEGEN_ENGINE]
# severity = "error"
`

const tomlInput = ref(DEFAULT_TOML)
const showConfig = ref(false)
const configError = ref('')

const fixDisabled = computed(() => !version.value)
const charCount = computed(() => {
  const len = input.value.length
  return `${len} char${len === 1 ? '' : 's'}`
})
const warningCount = computed(() =>
  warnings.value.length === 0
    ? 'no issues'
    : `${warnings.value.length} issue${warnings.value.length === 1 ? '' : 's'}`,
)

let linter: unknown = null
let debounceTimer: ReturnType<typeof setTimeout> | undefined
let tomlDebounceTimer: ReturnType<typeof setTimeout> | undefined
let wasmMod: Record<string, any> | null = null

// Vite refuses to `import()` files served from `public/` (they're copied as-is
// during build and never pass through the module graph). Bypass Vite's static
// analysis so the browser performs a native dynamic import of the URL instead.
const nativeImport = new Function('url', 'return import(url)') as (
  url: string,
) => Promise<Record<string, any>>

async function loadWasm(): Promise<Record<string, any>> {
  try {
    return await nativeImport(WASM_URL)
  } catch (e) {
    throw new Error(
      `${WASM_URL} is missing. Build it with "mise run build-wasm && mise run serve-local-wasm".`,
    )
  }
}

async function main(): Promise<void> {
  try {
    const mod = await loadWasm()
    await mod.default()
    version.value = mod.Linter.get_version()
    wasmMod = mod
    rebuildLinter()
    input.value = EXAMPLES.common
    lint()
    loading.value = false
  } catch (err) {
    loadError.value = `Failed to load the WebAssembly module: ${(err as Error).message}`
    loading.value = false
  }
}

/// (Re)build the linter from the TOML panel content. Returns false and keeps
/// the previous linter when the TOML is invalid.
function rebuildLinter(): boolean {
  if (!wasmMod) return false
  const text = tomlInput.value.trim()
  try {
    linter = text ? wasmMod.Linter.from_toml(tomlInput.value) : new wasmMod.Linter({})
    configError.value = ''
    return true
  } catch (err) {
    configError.value = `Invalid .malt.toml: ${typeof err === 'string' ? err : (err as Error).message}`
    return false
  }
}

function onTomlChange(): void {
  clearTimeout(tomlDebounceTimer)
  tomlDebounceTimer = setTimeout(() => {
    if (rebuildLinter()) lint()
  }, 600)
}

function resetToml(): void {
  tomlInput.value = DEFAULT_TOML
  if (rebuildLinter()) lint()
}

function lint(): void {
  if (!linter) return
  warnings.value = JSON.parse((linter as any).check(input.value, null))
}

function onInput(): void {
  clearTimeout(debounceTimer)
  debounceTimer = setTimeout(lint, 300)
}

function applyFixes(): void {
  if (!linter) return
  input.value = (linter as any).fix(input.value, null)
  lint()
}

function clearAll(): void {
  input.value = ''
  exampleKey.value = ''
  lint()
}

function loadExample(): void {
  if (exampleKey.value && EXAMPLES[exampleKey.value]) {
    input.value = EXAMPLES[exampleKey.value]
    lint()
  }
}

onMounted(main)
onBeforeUnmount(() => {
  clearTimeout(debounceTimer)
  clearTimeout(tomlDebounceTimer)
})
</script>

<template>
  <div id="malt-playground">
    <div v-if="loading" class="pg-status pg-loading">Loading malt-wasm…</div>
    <div v-else-if="loadError" class="pg-status pg-error">{{ loadError }}</div>

    <template v-else>
      <div class="pg-toolbar">
        <span class="pg-version">malt v{{ version }}</span>
        <div class="pg-toolbar-right">
          <select v-model="exampleKey" class="pg-select" @change="loadExample">
            <option value="">Load an example…</option>
            <option value="common">Common issues</option>
            <option value="prealloc">Loop preallocation</option>
            <option value="class">Class design</option>
            <option value="clean">Clean document</option>
          </select>
          <button class="pg-btn pg-btn-primary" :disabled="fixDisabled" @click="applyFixes">
            Fix all
          </button>
          <button class="pg-btn" @click="clearAll">Clear</button>
        </div>
      </div>

      <div class="pg-config">
        <button class="pg-btn pg-config-toggle" @click="showConfig = !showConfig">
          {{ showConfig ? '▾' : '▸' }} .malt.toml configuration
        </button>
        <div v-if="showConfig" class="pg-config-body">
          <MonacoEditor
            v-model="tomlInput"
            language="toml"
            min-height="220px"
            @change="onTomlChange"
          />
          <div class="pg-config-footer">
            <span v-if="configError" class="pg-config-error">{{ configError }}</span>
            <span v-else class="pg-meta">Applied automatically</span>
            <button class="pg-btn" @click="resetToml">Reset</button>
          </div>
        </div>
      </div>

      <div class="pg-panels">
        <div class="pg-panel">
          <div class="pg-panel-header">
            <span>MATLAB input</span>
            <span class="pg-meta">{{ charCount }}</span>
          </div>
          <MonacoEditor v-model="input" language="matlab" @change="onInput" />
        </div>
        <div class="pg-panel">
          <div class="pg-panel-header">
            <span>Warnings</span>
            <span class="pg-meta">{{ warningCount }}</span>
          </div>
          <div class="pg-results">
            <div v-if="warnings.length === 0" class="pg-empty pg-clean">No issues found</div>
            <div
              v-for="(w, i) in warnings"
              :key="`${w.line}:${w.column}:${w.rule_id}:${i}`"
              class="pg-warning"
            >
              <div class="pg-warning-header">
                <span class="pg-rule-badge">{{ w.rule_id || 'unknown' }}</span>
                <span class="pg-severity" :class="w.severity">{{ w.severity }}</span>
                <span class="pg-location">Line {{ w.line }}:{{ w.column }}</span>
                <span v-if="w.fix" class="pg-fix-badge">auto-fix</span>
              </div>
              <div class="pg-warning-message">{{ w.message }}</div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
#malt-playground {
  margin-top: 1rem;
  font-family: var(--vp-font-family-base, inherit);
}

.pg-status {
  padding: 1rem;
  border-radius: 4px;
  font-size: 0.85rem;
  text-align: center;
}
.pg-loading {
  background: var(--vp-code-block-bg, #f5f5f5);
  color: var(--vp-c-text-2, #666);
}
.pg-error {
  background: #fff0f0;
  color: #c0392b;
  border: 1px solid #f5c6cb;
}

.pg-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
  flex-wrap: wrap;
}
.pg-toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}
.pg-version {
  font-size: 0.75rem;
  color: var(--vp-c-text-2, #888);
  font-family: var(--vp-font-family-mono, monospace);
}

.pg-select {
  font-size: 0.8rem;
  padding: 0.3rem 0.5rem;
  border: 1px solid var(--vp-c-divider, #ddd);
  border-radius: 4px;
  background: var(--vp-c-bg, #fff);
  color: var(--vp-c-text-1, #333);
  cursor: pointer;
}

.pg-btn {
  font-size: 0.8rem;
  padding: 0.35rem 0.85rem;
  border: 1px solid var(--vp-c-divider, #ddd);
  border-radius: 4px;
  background: var(--vp-c-bg, #fff);
  color: var(--vp-c-text-1, #333);
  cursor: pointer;
  transition: opacity 0.15s;
}
.pg-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.pg-btn-primary {
  background: var(--vp-c-brand-1, #1565c0);
  color: var(--vp-c-bg, #fff);
  border-color: transparent;
}
.pg-btn-primary:not(:disabled):hover {
  opacity: 0.85;
}

.pg-panels {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}
@media (max-width: 768px) {
  .pg-panels {
    grid-template-columns: 1fr;
  }
}

.pg-panel {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--vp-c-divider, #ddd);
  border-radius: 6px;
  overflow: hidden;
  background: var(--vp-c-bg, #fff);
  min-height: 460px;
}

.pg-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
  background: var(--vp-code-block-bg, #f5f5f5);
  border-bottom: 1px solid var(--vp-c-divider, #ddd);
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--vp-c-text-2, #555);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.pg-meta {
  font-weight: 400;
  font-size: 0.75rem;
  color: var(--vp-c-text-2, #888);
  text-transform: none;
  letter-spacing: 0;
}

.pg-config {
  margin-bottom: 0.75rem;
}
.pg-config-toggle {
  width: 100%;
  text-align: left;
  font-family: var(--vp-font-family-mono, monospace);
}
.pg-config-body {
  margin-top: 0.5rem;
  border: 1px solid var(--vp-c-divider, #ddd);
  border-radius: 6px;
  overflow: hidden;
  background: var(--vp-c-bg, #fff);
}
.pg-config-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
  border-top: 1px solid var(--vp-c-divider, #ddd);
}
.pg-config-error {
  font-size: 0.8rem;
  color: #c0392b;
}

.pg-monaco {
  flex: 1;
}

.pg-results {
  flex: 1;
  min-height: 360px;
  overflow-y: auto;
  padding: 0.5rem;
}

.pg-warning {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  padding: 0.5rem 0.6rem;
  border-radius: 4px;
  margin-bottom: 0.4rem;
  border-left: 3px solid var(--vp-c-accent, #f57f17);
  background: var(--vp-code-block-bg, #fafafa);
  font-size: 0.82rem;
}
.pg-warning:last-child {
  margin-bottom: 0;
}

.pg-warning-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.pg-rule-badge {
  font-family: var(--vp-font-family-mono, monospace);
  font-size: 0.72rem;
  font-weight: 700;
  padding: 0.1rem 0.4rem;
  border-radius: 3px;
  background: var(--vp-c-brand-1, #1565c0);
  color: var(--vp-c-bg, #fff);
  white-space: nowrap;
}

.pg-severity {
  font-size: 0.68rem;
  padding: 0.1rem 0.35rem;
  border-radius: 3px;
  background: #e8f5e9;
  color: #2e7d32;
}
.pg-severity.Error {
  background: #fdecea;
  color: #c0392b;
}
.pg-severity.Info {
  background: #e3f2fd;
  color: #1565c0;
}

.pg-location {
  font-family: var(--vp-font-family-mono, monospace);
  font-size: 0.72rem;
  color: var(--vp-c-text-2, #888);
}

.pg-fix-badge {
  font-size: 0.68rem;
  padding: 0.1rem 0.35rem;
  border-radius: 3px;
  background: #fff3e0;
  color: #e65100;
  margin-left: auto;
}

.pg-warning-message {
  color: var(--vp-c-text-1, #333);
  line-height: 1.4;
}

.pg-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 200px;
  color: var(--vp-c-text-2, #888);
  font-size: 0.85rem;
  text-align: center;
  padding: 1rem;
}

.pg-clean {
  color: #2e7d32;
  font-weight: 500;
}
</style>
