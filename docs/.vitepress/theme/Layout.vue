<script setup lang="ts">
import DefaultTheme from 'vitepress/theme'
import { useData } from 'vitepress'
import PageIcon from './components/PageIcon.vue'

const { frontmatter } = useData()
</script>

<template>
  <DefaultTheme.Layout>
    <template #doc-before>
      <PageIcon :icon="frontmatter.icon" />
    </template>
  </DefaultTheme.Layout>
</template>
