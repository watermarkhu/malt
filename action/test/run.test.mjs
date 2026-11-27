// The whole run against the real WebAssembly build. Needs `mise run build-wasm` and
// `bun action/build.mjs` first; skipped when the vendored glue is missing.
import { afterEach, describe, expect, test } from 'bun:test'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const glue = path.join(here, '..', 'src', 'vendor', 'malt_lib.js')
const binary = path.join(here, '..', 'dist', 'malt_lib_bg.wasm')
const ready = fs.existsSync(glue) && fs.existsSync(binary)

let wasm = null
let run = null
if (ready) {
  const module = await import(glue)
  await module.default({ module_or_path: fs.readFileSync(binary) })
  wasm = module
  run = (await import('../src/run.mjs')).run
}

const dirs = []
afterEach(() => {
  for (const dir of dirs.splice(0)) fs.rmSync(dir, { recursive: true, force: true })
})

/** A workspace with `files` (name to content) and the files the runner provides. */
function workspace(files) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'malt-run-'))
  dirs.push(dir)
  for (const [name, content] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(dir, name)), { recursive: true })
    fs.writeFileSync(path.join(dir, name), content)
  }
  return dir
}

function go(dir, inputs = {}) {
  const lines = []
  const env = {
    GITHUB_WORKSPACE: dir,
    GITHUB_STEP_SUMMARY: path.join(dir, '.summary.md'),
    GITHUB_OUTPUT: path.join(dir, '.output.txt'),
  }
  for (const [name, value] of Object.entries(inputs)) env[`INPUT_${name.toUpperCase()}`] = value
  const outcome = run({ wasm, env, cwd: dir, out: { write: (text) => lines.push(text) } })
  return { outcome, log: lines.join(''), env }
}

// NOPTS (info) on line 1, EOLPAR (error) on line 2.
const BAD = 'x = 1\nfunction f(\n'

describe.skipIf(!ready)('run', () => {
  test('annotates, summarizes, counts and fails on errors', () => {
    const dir = workspace({ 'a.m': BAD })
    const { outcome, log, env } = go(dir)
    expect(log).toContain('::notice file=a.m,line=1,col=3,title=NOPTS::')
    expect(log).toContain('::error file=a.m,line=2,col=11,title=EOLPAR::')
    expect(outcome).toMatchObject({ findings: 2, errors: 1, warnings: 0, files: 1, failed: true })
    expect(fs.readFileSync(env.GITHUB_STEP_SUMMARY, 'utf8')).toStartWith('## malt')
    expect(fs.readFileSync(env.GITHUB_OUTPUT, 'utf8')).toContain('findings=2\nerrors=1\nwarnings=0\n')
  })

  test('fail-on decides the exit, level the reporting', () => {
    const dir = workspace({ 'a.m': 'x = 1\n' })
    expect(go(dir).outcome).toMatchObject({ findings: 1, failed: false })
    expect(go(dir, { 'fail-on': 'info' }).outcome.failed).toBe(true)
    expect(go(dir, { level: 'warning', 'fail-on': 'info' }).outcome).toMatchObject({ findings: 0, failed: false })
    expect(go(workspace({ 'a.m': BAD }), { 'fail-on': 'never' }).outcome.failed).toBe(false)
  })

  test('annotations and summary can be switched off', () => {
    const dir = workspace({ 'a.m': BAD })
    const { log, env } = go(dir, { annotations: 'false', summary: 'false' })
    expect(log).not.toContain('::error file=')
    expect(fs.existsSync(env.GITHUB_STEP_SUMMARY)).toBe(false)
  })

  test('the preset input overrules .malt.toml', () => {
    const files = { 'a.m': 'function f(x)\n  y = 1;\nend\n', '.malt.toml': '[lint]\npreset = "mathworks"\n' }
    const dir = workspace(files)
    expect(go(dir).log).not.toContain('MP_')
    expect(go(dir, { preset: 'monkeyproof' }).log).toContain('title=MP_')
  })

  test('the toml decides which files are found', () => {
    const dir = workspace({ 'a.m': BAD, 'gen/g.m': BAD, '.malt.toml': '[lint]\nextend_exclude = ["gen/"]\n' })
    expect(go(dir).outcome.files).toBe(1)
  })

  test('paths narrow the run, a missing one is a NOFIL error', () => {
    const dir = workspace({ 'src/a.m': 'x = 1;\n', 'other/b.m': BAD })
    expect(go(dir, { paths: 'src' }).outcome).toMatchObject({ findings: 0, files: 1 })
    const { outcome, log } = go(dir, { paths: 'src missing.m' })
    expect(outcome).toMatchObject({ errors: 1, failed: true })
    expect(log).toContain('title=NOFIL::Unable to open file missing.m')
  })

  test('output-json holds the version, the configuration and the findings', () => {
    const dir = workspace({ 'a.m': BAD, '.malt.toml': '[lint]\nextend_exclude = ["gen/"]\n' })
    const { env } = go(dir, { 'output-json': 'results.json' })
    const results = JSON.parse(fs.readFileSync(path.join(dir, 'results.json'), 'utf8'))
    expect(results.schema_version).toBe(1)
    expect(results.malt_version).toBe(wasm.Linter.get_version())
    expect(results.config.config_file).toBe('.malt.toml')
    expect(results.config.exclude).toContain('gen/')
    expect(results.files).toEqual(['a.m'])
    expect(results.findings.map((f) => f.rule)).toEqual(['NOPTS', 'EOLPAR'])
    expect(fs.readFileSync(env.GITHUB_OUTPUT, 'utf8')).toContain(`results-file=${path.join(dir, 'results.json')}`)
  })

  test('an empty workspace warns and passes', () => {
    const { outcome, log } = go(workspace({}))
    expect(outcome).toMatchObject({ findings: 0, files: 0, failed: false })
    expect(log).toContain('::warning::malt found no MATLAB files')
  })

  test('a file that is not UTF-8 is read as Latin-1', () => {
    const dir = workspace({})
    fs.writeFileSync(path.join(dir, 'l.m'), Buffer.from('% caf\xe9\nx = 1;\n', 'latin1'))
    expect(go(dir).outcome.files).toBe(1)
  })

  test('config warnings become warning annotations', () => {
    const dir = workspace({ 'a.m': 'x = 1;\n', '.malt.toml': '[lint.rules]\nNOSUCHRULE = "off"\n' })
    expect(go(dir).log).toContain('::warning file=.malt.toml')
  })
})
