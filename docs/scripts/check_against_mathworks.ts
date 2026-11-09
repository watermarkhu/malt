// Compare malt's declared checks with the official MathWorks index.
//
//   bun docs/scripts/check_against_mathworks.ts ~/.local/share/malt-reference/mathworks-checks.csv
//
// The CSV is MathWorks documentation content and is not part of this repository;
// see .agents/skills/rule-pipeline/SKILL.md for how to regenerate it. The script
// reads the check tables in the `//!` docstrings of crates/malt_rules/src and the
// `id = "..."` entries of the data files, and reports:
//   1. IDs we declare that MathWorks does not define,
//   2. IDs MathWorks defines that we do not declare,
//   3. severity differences,
//   4. message differences (after normalising whitespace, quotes and placeholders).
// It exits 1 when 1 or 3 is non-empty.
import { readdirSync, readFileSync, statSync } from 'fs'
import { join } from 'path'

const csvPath = Bun.argv[2]
if (!csvPath) { console.error('usage: bun check_against_mathworks.ts <mathworks-checks.csv>'); process.exit(2) }

function parseCsv(text: string): string[][] {
  const rows: string[][] = []; let row: string[] = []; let cell = ''; let quoted = false
  for (let i = 0; i < text.length; i++) {
    const c = text[i]
    if (quoted) { if (c === '"') { if (text[i + 1] === '"') { cell += '"'; i++ } else quoted = false } else cell += c }
    else if (c === '"') quoted = true
    else if (c === ',') { row.push(cell); cell = '' }
    else if (c === '\n') { row.push(cell); rows.push(row); row = []; cell = '' }
    else cell += c
  }
  return rows.slice(1)
}

type Official = { category: string; severity: string; message: string }
const official = new Map<string, Official>()
for (const r of parseCsv(readFileSync(csvPath, 'utf8'))) {
  if (r.length >= 4 && !official.has(r[1])) official.set(r[1], { category: r[0], severity: r[2].toLowerCase(), message: r[3] })
}

const src = join(import.meta.dir, '../../crates/malt_rules/src')
type Ours = { severity?: string; message?: string; file: string }
const ours = new Map<string, Ours>()
const walk = (d: string) => {
  for (const f of readdirSync(d)) {
    const p = join(d, f)
    if (statSync(p).isDirectory()) walk(p)
    else if (p.endsWith('.rs')) {
      for (const m of readFileSync(p, 'utf8').matchAll(/^\/\/! \| ([A-Za-z][A-Za-z0-9_.]*) \| (error|warning|info) \|[^|]*\| ?(.*?) \|\s*$/gm))
        ours.set(m[1], { severity: m[2], message: m[3].trim(), file: p.replace(src + '/', '') })
    } else if (p.endsWith('.toml')) {
      for (const m of readFileSync(p, 'utf8').matchAll(/^id = "([A-Za-z][A-Za-z0-9_.]*)"/gm))
        if (!ours.has(m[1])) ours.set(m[1], { file: p.replace(src + '/', '') })
    }
  }
}
walk(src)

const norm = (s: string) => s.toLowerCase().replace(/[\s'"`]+/g, ' ').trim()
const unknown = [...ours.keys()].filter(i => !official.has(i))
// naming.* IDs are generated at runtime; NOFIL/RDERR live in malt_cli, QUIT in
// malt_core and NOSEMI is a standalone rule without a check table.
const dynamic = (i: string) => i.startsWith('naming.') || ['NOFIL', 'RDERR', 'QUIT', 'NOSEMI'].includes(i)
const missing = [...official.keys()].filter(i => !ours.has(i) && !dynamic(i))
const sev = [...ours].filter(([i, o]) => o.severity && official.has(i) && official.get(i)!.severity !== o.severity)
const msg = [...ours].filter(([i, o]) => o.message && official.has(i) && !/var_/i.test(o.message) && norm(o.message) !== norm(official.get(i)!.message))

console.log(`official IDs: ${official.size}, declared: ${ours.size}`)
console.log(`\n1. declared but not official (${unknown.length})\n` + unknown.map(i => `  ${i} [${ours.get(i)!.file}]`).join('\n'))
console.log(`\n2. official but not declared (${missing.length})`)
const byCat = new Map<string, string[]>()
for (const i of missing) byCat.set(official.get(i)!.category, [...(byCat.get(official.get(i)!.category) || []), i])
for (const [c, l] of byCat) console.log(`  ${c} (${l.length}): ${l.join(' ')}`)
console.log(`\n3. severity differences (${sev.length})\n` + sev.map(([i, o]) => `  ${i}: ours ${o.severity}, official ${official.get(i)!.severity} [${o.file}]`).join('\n'))
console.log(`\n4. message differences (${msg.length})\n` + msg.map(([i, o]) => `  ${i}\n    ours:     ${o.message}\n    official: ${official.get(i)!.message}`).join('\n'))
process.exit(unknown.length || sev.length ? 1 : 0)
