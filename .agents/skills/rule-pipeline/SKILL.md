---
name: rule-pipeline
description: Orchestrates implementation of MATLAB Code Analyzer rules in malt: plan in the main session, then review and implement via parallel Reviewer → Implementer subagent flows. Use when implementing one or more new lint rules or checks in this repo, or when asked to run the rule pipeline / per-rule flow. Launches many independent flows in parallel, one per rule.
---

# Rule Implementation Pipeline (plan in main → Review → Implement)

Every new MATLAB Code Analyzer rule in `malt` is implemented as follows:

1. **Planning happens in the MAIN session** (not a subagent) — the main session owns
   the single Playwright MCP instance and does the rule-context lookup and codebase
   study, then writes the implementation plan.
2. **Review** — one `reviewer` subagent per rule, launched in parallel.
3. **Implement** — one `implementer` subagent per rule, launched in parallel.

Flows for different rules are fully independent and run in **parallel** for steps 2
and 3.

## The agents

| Step | Where | Subagent (`subagent_type`) | Model | Output |
|------|-------|---------------------------|-------|--------|
| 1. Plan | main session | — | `opencode-go/deepseek-v4-flash` (current) | Implementation plan |
| 2. Review | subagent | `reviewer` | `opencode-go/mimo-v2.5` | Verdict + corrected plan |
| 3. Implement | subagent | `implementer` | `opencode-go/deepseek-v4-flash` | Code, tests, docs + verification |

Agent prompts live in `.agents/agents/{reviewer,implementer}.md`; their description,
mode, model, and permissions are registered in the root `opencode.json`.

## Rule context lookup (main session only)

Before planning, look the rule up in the official index. MathWorks returns HTTP 403 to
`webfetch` and the Playwright MCP needs a Chrome that is not installed here, so use the
local copy: `~/.local/share/malt-reference/mathworks-checks.csv` (see its `make-csv.ts` to
regenerate it from the Wayback Machine snapshot of
`https://www.mathworks.com/help/matlab/matlab_env/index-of-code-analyzer-checks.html`).
The CSV has the ID, category, severity, message and whether the check can be disabled.
It does not have code examples; derive MUST/MUST NOT examples from the message.

Subagents must not browse; pass the rule context to them as plan text.

## Orchestration

Given a list of rule IDs (e.g. the remaining readability checks COMNL, STLOW, FLUDLR,
MFAMB, FVINR), run one flow per rule:

1. **Plan in the main session.** For each rule:
   - Look up the rule context on the MathWorks check index via Playwright (see above).
   - Study the codebase conventions: `.agents/AGENTS.md` "Adding a New Rule", the target
     module in `crates/malt_rules/src/`, a reference rule like `nosemi.rs`, and
     `docs/nosemi.md`.
   - Write a self-contained implementation plan including: rule ID, official
     description, severity, category, can_be_disabled; `target_node_types()` or
     file-level `check_file`; concrete detection logic with the tree-sitter node
     kinds involved and how to distinguish the pattern from lookalikes (remember
     `function_call` is also used for indexing); MATLAB examples that MUST fire and
     MUST NOT fire; exact files to create/modify (rule module, `lib.rs` `pub mod`,
      `docs/<id>.md`, `docs/rules.md` row, VitePress sidebar (`docs/.vitepress/config.mts`)); any rule-specific
     config params; tests to add; pitfalls.
2. **Launch ALL Reviewers in parallel** — one `task` call per plan, all in a single
   message, `subagent_type: "reviewer"`, prompt includes the rule ID, the category,
   and the plan text. Do not wait between them.
3. **Collect the verdicts**, then **launch ALL Implementers in parallel** — one per
   approved/corrected plan, `subagent_type: "implementer"`, prompt includes the rule
   ID, the category, and the plan text. Skip a rule only if the Reviewer rejects it
   outright with no usable plan; otherwise pass the corrected plan through.

Do not serialize steps 2 and 3. Each is a batch of parallel tasks.

## Completion criteria per rule

- `cargo build` succeeds
- `cargo clippy --all-targets` passes with zero warnings
- `cargo test` passes (fires, not-fires, and config tests from the plan)
- Docs updated: `docs/<rule_id>.md`, `docs/rules.md`, VitePress sidebar (`docs/.vitepress/config.mts`)
- `crates/malt_rules/src/lib.rs` has the `pub mod` line for any new module

## Notes

- Do not commit changes unless the user explicitly asks.
- If the tree-sitter grammar cannot represent the pattern (e.g. `function_call` also
  covers indexing), the plan/review must address the ambiguity explicitly.

## Running audits at scale (engine-wide remaps)

When a whole engine is audited rather than one new rule:

- One agent per engine directory (split large engines by file group), in batches of
  **at most three** at a time, so a usage limit cannot strand half-edited files.
- Each agent edits only its own files, never commits, and runs rustfmt on its files
  only (not `cargo fmt --all`). A build error in someone else's file means wait and retry.
- Agents must leave the tree compiling. Verify the combined tree yourself
  (`cargo test --workspace --locked`, clippy, docs check) before committing.
- Commit once per coherent chunk: hooks check the whole tree, so per-engine commits fail
  while other engines are mid-edit.
- Tell each agent which IDs other engines own, so a duplicate is deleted in one place
  only (the audits of readability and performance both deleted PSIZE).
