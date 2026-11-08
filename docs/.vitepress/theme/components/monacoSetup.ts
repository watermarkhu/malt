/// <reference types="vite/client" />

// Shared, lazily-loaded Monaco + Shiki setup for the docs playground.
//
// Everything is imported dynamically so neither Monaco nor Shiki ends up in
// the initial page bundle (or the SSR build): the playground loads them on
// first mount, client-side only.
import type * as monacoNs from 'monaco-editor'

let setup: Promise<typeof monacoNs> | null = null

export function loadMonaco(): Promise<typeof monacoNs> {
  if (!setup) {
    setup = (async () => {
      const monaco = await import('monaco-editor')

      // Monaco's ESM build loads its editor worker from a generated URL when
      // no custom factory is set, which 404s under VitePress (dev and build).
      // Provide a minimal inline worker instead: the playground uses no Monaco
      // language services (tokenization comes from Shiki on the main thread),
      // so the worker never needs to answer.
      ;(globalThis as unknown as { MonacoEnvironment?: unknown }).MonacoEnvironment = {
        getWorker: () =>
          new Worker(
            URL.createObjectURL(
              new Blob(['self.onmessage = () => {}'], { type: 'text/javascript' }),
            ),
          ),
      }

      const { createHighlighter } = await import('shiki')
      const { createJavaScriptRegexEngine } = await import('shiki/engine/javascript')
      const { shikiToMonaco } = await import('@shikijs/monaco')

      const highlighter = await createHighlighter({
        themes: ['vitesse-light', 'vitesse-dark'],
        langs: ['matlab', 'toml'],
        engine: createJavaScriptRegexEngine(),
      })

      // shikiToMonaco only wires token providers for languages already
      // registered with Monaco; neither matlab nor toml is built in.
      for (const id of ['matlab', 'toml'] as const) {
        if (!monaco.languages.getLanguages().some((l) => l.id === id)) {
          monaco.languages.register({ id })
        }
      }
      shikiToMonaco(highlighter, monaco)
      return monaco
    })()
  }
  return setup
}

export function monacoThemeFor(dark: boolean): string {
  return dark ? 'vitesse-dark' : 'vitesse-light'
}
