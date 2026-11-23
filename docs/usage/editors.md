---
icon: lucide/code
---

# Editor integration

::: info Coming Soon

Editor integration is planned for a future release.

:::

## Planned integrations

### Language Server Protocol (LSP)

An LSP server for malt is on the roadmap. It will report diagnostics as you type, offer quick-fix code actions, and work with any LSP-compatible editor.

### VS Code

A VS Code extension is planned. It will wrap the LSP server, highlight diagnostics inline, apply fixes through code actions and offer snippets for `.malt.toml`.

### Neovim

Once the LSP server exists, Neovim users can set it up with `nvim-lspconfig` or a similar plugin.

### MATLAB Editor

The built-in MATLAB editor is being explored.

## Current workaround

Until then, run malt as an external tool:

1. Configure your editor to run `malt <current-file>` on save
2. Parse the output format (`file:line:column [severity] rule: message`) for diagnostics
3. Many editors support this via "problem matcher" patterns

## Large files

malt refuses files over `max_file_size` (5 MiB by default; 2 MiB in the browser build) and code nested beyond its depth limits: the file gets a `MALT_TOO_BIG` or `MALT_TOO_DEEP` error at line 1 and is not analyzed, so a huge generated file cannot freeze the editor. Raising these limits in `[lint.limits]` can make malt use gigabytes of memory on such files; see [Resource limits](../configuration.md#resource-limits) before you do.

## Contributing

To help build editor integrations, see the contribution guidelines in the [repository](https://github.com/watermarkhu/malt).
