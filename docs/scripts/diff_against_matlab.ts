// Compare what malt reports with what MATLAB's Code Analyzer reports, per file and line.
//
//   bun docs/scripts/diff_against_matlab.ts <files.txt> <matlab.tsv> [--preset all] [--malt target/release/malt] [--detail]
//
// <files.txt>   absolute paths of the .m files, one per line (the same list given to matlab_probe)
// <matlab.tsv>  output of docs/scripts/matlab_probe.m (path, line, column, id, message)
//
// Both sides are reduced to (file, line, id) triples. The report lists, per check ID:
//   both        reported by MATLAB and malt on the same line
//   matlab-only reported by MATLAB, missed by malt (a candidate miss)
//   malt-only   reported by malt, not by MATLAB (a false-positive candidate)
// With --detail every differing triple is printed. Exit code 1 when there are differences.
//
// The MATLAB output is MathWorks content: keep it outside the repository.
import { readFileSync } from 'fs'

const args = Bun.argv.slice(2)
const flag = (name: string, dflt: string) => { const i = args.indexOf(name); return i >= 0 ? args[i + 1] : dflt }
const [listPath, tsvPath] = args.filter((a, i) => !a.startsWith('--') && !(i > 0 && args[i - 1].startsWith('--') && args[i - 1] !== '--detail'))
if (!listPath || !tsvPath) { console.error('usage: bun diff_against_matlab.ts <files.txt> <matlab.tsv> [--preset all] [--malt path] [--detail]'); process.exit(2) }
const preset = flag('--preset', 'all')
const maltBin = flag('--malt', 'target/release/malt')
const detail = args.includes('--detail')

const files = readFileSync(listPath, 'utf8').split('\n').map(s => s.trim()).filter(Boolean)
const fileSet = new Set(files)
const key = (f: string, line: number | string, id: string) => `${f}\t${line}\t${id}`

const matlab = new Map<string, string>()
for (const row of readFileSync(tsvPath, 'utf8').split('\n')) {
  const [f, line, , id, msg] = row.split('\t')
  // MATLAB reports file-level findings (BDFIL, ...) at line 0; malt reports them at line 1.
  if (f && id && fileSet.has(f)) matlab.set(key(f, line === '0' ? '1' : line, id), msg ?? '')
}

const malt = new Map<string, string>()
for (let i = 0; i < files.length; i += 100) {
  const batch = files.slice(i, i + 100)
  const out = Bun.spawnSync([maltBin, '--preset', preset, ...batch], { stdout: 'pipe', stderr: 'pipe' }).stdout.toString()
  for (const l of out.split('\n')) {
    const m = l.match(/^(.*?):(\d+):\d+ \[[EWI]\] ([A-Za-z][A-Za-z0-9_.]*): (.*)$/)
    if (m) malt.set(key(m[1], m[2], m[3]), m[4])
  }
}

type Tally = { both: string[]; matlabOnly: string[]; maltOnly: string[] }
const byId = new Map<string, Tally>()
const tally = (id: string) => { let t = byId.get(id); if (!t) byId.set(id, t = { both: [], matlabOnly: [], maltOnly: [] }); return t }
const idOf = (k: string) => k.split('\t')[2]
const shown = (k: string) => { const [f, line, id] = k.split('\t'); return `${f.replace(/^.*\/fixtures\//, '')}:${line} ${id}` }
for (const k of matlab.keys()) (malt.has(k) ? tally(idOf(k)).both : tally(idOf(k)).matlabOnly).push(k)
for (const k of malt.keys()) if (!matlab.has(k)) tally(idOf(k)).maltOnly.push(k)

const rows = [...byId.entries()].sort((a, b) => (b[1].matlabOnly.length + b[1].maltOnly.length) - (a[1].matlabOnly.length + a[1].maltOnly.length))
let nBoth = 0, nMissed = 0, nExtra = 0
console.log(`files: ${files.length}, MATLAB findings: ${matlab.size}, malt findings (--preset ${preset}): ${malt.size}\n`)
console.log('id'.padEnd(14) + 'both'.padStart(6) + 'matlab-only'.padStart(13) + 'malt-only'.padStart(11))
for (const [id, t] of rows) {
  nBoth += t.both.length; nMissed += t.matlabOnly.length; nExtra += t.maltOnly.length
  if (t.matlabOnly.length + t.maltOnly.length === 0) continue
  console.log(id.padEnd(14) + String(t.both.length).padStart(6) + String(t.matlabOnly.length).padStart(13) + String(t.maltOnly.length).padStart(11))
  if (detail) {
    for (const k of t.matlabOnly) console.log(`    matlab-only  ${shown(k)}  ${matlab.get(k)}`)
    for (const k of t.maltOnly) console.log(`    malt-only    ${shown(k)}  ${malt.get(k)}`)
  }
}
console.log(`\ntotal: both ${nBoth}, matlab-only ${nMissed}, malt-only ${nExtra}`)
process.exit(nMissed + nExtra > 0 ? 1 : 0)
