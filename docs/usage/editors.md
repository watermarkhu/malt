---
icon: lucide/code
---

# Editor Integration

::: info Coming Soon

Editor integration is planned for a future release. This page will be updated with instructions as support is added.

:::

## Planned Integrations

### Language Server Protocol (LSP)

An LSP server for malt is on the roadmap. Once available, it will provide:

- Real-time diagnostics as you type
- Quick-fix code actions
- Integration with any LSP-compatible editor

### VS Code

A VS Code extension is planned that will wrap the malt LSP server and provide:

- Inline diagnostic highlighting
- One-click auto-fix via code actions
- Configuration snippets for `.malt.toml`

### Neovim

Once the LSP server is available, Neovim users can configure it via `nvim-lspconfig` or similar plugins.

### MATLAB Editor

Integration with the built-in MATLAB editor is being explored.

## Current Workaround

Until native editor integration is available, you can use malt as an external tool:

1. Configure your editor to run `malt <current-file>` on save
2. Parse the output format (`file:line:column [severity] rule: message`) for diagnostics
3. Many editors support this via "problem matcher" patterns

## Large files

malt refuses files over `max_file_size` (5 MiB by default; 2 MiB in the browser build) and code nested beyond its depth limits: the file gets a `MALT_TOO_BIG` or `MALT_TOO_DEEP` error at line 1 and is not analyzed, so a huge generated file cannot freeze the editor. Raising these limits in `[lint.limits]` can make malt use gigabytes of memory on such files; see [Resource limits](../configuration.md#resource-limits) before you do.

## Contributing

If you'd like to help build editor integrations, see the [repository](https://github.com/watermarkhu/malt) for contribution guidelines.
