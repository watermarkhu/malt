import DefaultTheme from 'vitepress/theme'
import Layout from './Layout.vue'
import Playground from './components/Playground.vue'
import './style.css'

export default {
  extends: DefaultTheme,
  Layout,
  enhanceApp({ app }) {
    app.component('Playground', Playground)
  },
}
