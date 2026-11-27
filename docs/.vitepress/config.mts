import { defineConfig } from 'vitepress'
import { codeTitles } from './plugins/code-titles'

const cratesIconSvg =
  '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12 2.5 21 7v10l-9 4.5L3 17V7l9-4.5Zm0 2.2L5.3 8.1 12 11.3l6.7-3.2L12 4.7Zm-8 4.9v7.2l7 3.5v-7.2l-7-3.5Zm16 0-7 3.5v7.2l7-3.5V9.6Z" fill="currentColor"/></svg>'

export default defineConfig({
  lang: 'en-US',
  title: 'malt',
  description: 'An ultra-fast, extensible linter for MATLAB, written in Rust',
  cleanUrls: true,
  // GitHub Pages serves the project site at https://watermarkhu.nl/malt/.
  base: '/malt/',
  lastUpdated: true,

  head: [['link', { rel: 'icon', type: 'image/png', href: '/malt/favicon.png' }]],

  markdown: {
    config: (md) => {
      codeTitles(md)
    },
  },

  // The rules.md page embeds the full generated data-driven check tables, so its
  // page chunk legitimately approaches 500 kB.
  build: {
    chunkSizeWarningLimit: 5000,
  },

  themeConfig: {
    logo: '/logo.png',
    siteTitle: 'malt',
    nav: [
      { text: 'Getting Started', link: '/getting-started/installation', activeMatch: '/getting-started/' },
      { text: 'Usage', link: '/usage/cli', activeMatch: '/usage/' },
      { text: 'Playground', link: '/playground', activeMatch: '/playground' },
      { text: 'Configuration', link: '/configuration', activeMatch: '/configuration' },
      { text: 'Path index', link: '/api/', activeMatch: '/api/' },
      { text: 'Rules', link: '/rules', activeMatch: '/rules' },
      { text: 'Conformance', link: '/verification', activeMatch: '/verification|/migration' },
    ],
    sidebar: {
      '/': [
        {
          text: 'Getting Started',
          collapsed: false,
          items: [
            { text: 'Installation', link: '/getting-started/installation' },
            { text: 'Quick Start', link: '/getting-started/quickstart' },
          ],
        },
        {
          text: 'Usage',
          collapsed: false,
          items: [
            { text: 'CLI Reference', link: '/usage/cli' },
            { text: 'Editor Integration', link: '/usage/editors' },
            { text: 'CI/CD', link: '/usage/ci-cd' },
            { text: 'GitHub Action', link: '/usage/github-action' },
            { text: 'WebAssembly & npm', link: '/usage/wasm' },
          ],
        },
        { text: 'Playground', link: '/playground' },
        { text: 'Configuration', link: '/configuration' },
        {
          text: 'Path index',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/api/' },
            { text: 'Python API', link: '/api/python' },
            { text: 'Rust API', link: '/api/rust' },
            { text: 'How MATLAB is read', link: '/api/behaviour' },
            { text: 'Coming from maxx', link: '/api/maxx' },
            { text: 'Development', link: '/api/development' },
          ],
        },
        {
          text: 'Conformance',
          collapsed: false,
          items: [
            { text: 'How malt is verified', link: '/verification' },
            { text: 'Migration from earlier malt', link: '/migration' },
          ],
        },
        {
          text: 'Rules',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/rules' },
            {
              text: 'Error & Correctness',
              collapsed: true,
              items: [
                { text: 'Syntax Errors', link: '/rules/syntax-errors' },
                { text: 'Language Specification', link: '/rules/language-spec' },
                { text: 'Bugs', link: '/rules/bugs' },
                { text: 'Unset Variables', link: '/rules/unset-variables' },
                { text: 'Unused Constructions', link: '/rules/unused' },
                { text: 'Incomplete Analysis', link: '/rules/incomplete-analysis' },
              ],
            },
            {
              text: 'Style & Quality',
              collapsed: true,
              items: [
                { text: 'Good Practices', link: '/rules/good-practices' },
                { text: 'Readability Improvements', link: '/rules/readability' },
                { text: 'Formatting Suggestions', link: '/rules/formatting' },
                { text: 'Performance Improvements', link: '/rules/performance' },
                { text: 'Custom Checks', link: '/rules/custom-checks' },
                { text: 'Naming Checks', link: '/rules/naming' },
                { text: 'NOSEMI - Extra Semicolon', link: '/rules/nosemi' },
              ],
            },
            {
              text: 'Compatibility & Data-Driven',
              collapsed: true,
              items: [
                { text: 'Compatibility Considerations', link: '/rules/compatibility' },
                { text: 'Suggested Improvements', link: '/rules/suggested-improvements' },
              ],
            },
            {
              text: 'MonkeyProof standard',
              collapsed: true,
              items: [
                { text: 'Overview', link: '/monkeyproof' },
                { text: 'Naming conventions', link: '/rules/monkeyproof-naming' },
                { text: 'Layout & comments', link: '/rules/monkeyproof-layout' },
                { text: 'Statements & expressions', link: '/rules/monkeyproof-statements' },
                { text: 'Higher level rules', link: '/rules/monkeyproof-functions' },
                { text: 'Security', link: '/rules/monkeyproof-security' },
              ],
            },
            {
              text: 'Specialized',
              collapsed: true,
              items: [
                { text: 'MATLAB for Code Generation', link: '/rules/codegen' },
                { text: 'MATLAB Compiler (Deployment)', link: '/rules/deployment' },
                { text: 'System Objects', link: '/rules/system-objects' },
                { text: 'Unsupported Features', link: '/rules/unsupported' },
              ],
            },
          ],
        },
      ],
    },
    search: { provider: 'local' },
    editLink: {
      pattern: 'https://github.com/watermarkhu/malt/edit/main/docs/:path',
      text: 'Edit this page',
    },
    lastUpdated: { text: 'Updated' },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/watermarkhu/malt' },
      { icon: { svg: cratesIconSvg }, link: 'https://crates.io/crates/malt' },
    ],
    footer: {
      message: 'Copyright © 2026 Mark Shui Hu',
      copyright: 'malt is licensed under the MIT license',
    },
  },
})
