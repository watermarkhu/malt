<script setup lang="ts">
import { computed } from 'vue'
import {
  AlignLeft,
  AlertTriangle,
  Archive,
  Ban,
  BookMarked,
  Boxes,
  Braces,
  Bug,
  CheckCircle,
  Code,
  Cog,
  Download,
  Eye,
  GitBranch,
  Lightbulb,
  ListChecks,
  Package,
  Play,
  PlayCircle,
  Settings,
  SlidersHorizontal,
  Terminal,
  Trash2,
  Type,
  Wrench,
  XCircle,
  Zap,
  type LucideIcon,
} from 'lucide-vue-next'

const props = defineProps<{ icon?: string }>()

// Icons referenced by the `icon: lucide/...` page frontmatter. Named imports
// keep the bundle tree-shakeable (no `import *` wholesale bundle).
const ICONS: Record<string, LucideIcon> = {
  'align-left': AlignLeft,
  'alert-triangle': AlertTriangle,
  archive: Archive,
  ban: Ban,
  'book-marked': BookMarked,
  boxes: Boxes,
  braces: Braces,
  bug: Bug,
  'check-circle': CheckCircle,
  code: Code,
  cog: Cog,
  download: Download,
  eye: Eye,
  'git-branch': GitBranch,
  lightbulb: Lightbulb,
  'list-checks': ListChecks,
  package: Package,
  play: Play,
  'play-circle': PlayCircle,
  settings: Settings,
  'sliders-horizontal': SlidersHorizontal,
  terminal: Terminal,
  'trash-2': Trash2,
  type: Type,
  wrench: Wrench,
  'x-circle': XCircle,
  zap: Zap,
}

const Icon = computed(() => {
  const name = (props.icon || '').replace(/^lucide\//, '')
  return ICONS[name] ?? null
})
</script>

<template>
  <div v-if="Icon" class="page-icon">
    <component :is="Icon" :size="48" :stroke-width="2" />
  </div>
</template>
