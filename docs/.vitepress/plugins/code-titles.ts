type FenceRenderer = (
  tokens: unknown[],
  idx: number,
  options: unknown,
  env: unknown,
  self: { renderToken?: (tokens: unknown[], idx: number, options: unknown) => string },
) => string

type MarkdownItLike = {
  renderer: {
    rules: Record<string, FenceRenderer | undefined>
  }
  utils: { escapeHtml: (s: string) => string }
}

/**
 * markdown-it plugin that renders `title="..."` from a fenced code block's
 * info string as a `.vp-code-title` caption above the code block:
 *
 *     ```toml title=".malt.toml"
 *
 * The `title="..."` attribute is stripped from the info string so the code
 * block itself still gets the correct language class.
 */
export function codeTitles(md: MarkdownItLike): void {
  const fence = md.renderer.rules.fence
  md.renderer.rules.fence = (tokens, idx, options, env, self) => {
    const token = tokens[idx] as { info?: string }
    const info = token.info || ''
    const match = info.match(/\btitle="([^"]*)"/)
    if (!match || !fence) {
      return fence ? fence(tokens, idx, options, env, self) : ''
    }
    token.info = info.replace(/\btitle="[^"]*"/, '').trim()
    const title = md.utils.escapeHtml(match[1])
    return `<div class="vp-code-title">${title}</div>` + fence(tokens, idx, options, env, self)
  }
}
