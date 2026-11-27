// The shell of the action: all it does is input and output. It hands the raw inputs and three
// file system callbacks to the WebAssembly build (`run_action`), which does the work (reading
// `.malt.toml`, finding the files, linting, formatting), and writes back what it returns.

import fs from 'node:fs'
import path from 'node:path'

const INPUTS = ['paths', 'preset', 'level', 'fail-on', 'annotations', 'summary', 'output-json']

/** What a directory entry is, for the file search: links and other entries are skipped. */
function kindOf(entry) {
  if (entry.isDirectory()) return 'dir'
  if (entry.isFile()) return 'file'
  return 'other'
}

/**
 * Run the action once. `wasm` is the loaded malt-wasm module, `env` the environment, `cwd` the
 * workspace and `out` where the log goes. Returns the outcome of `run_action`.
 */
export function runShell({ wasm, env = process.env, cwd = process.cwd(), out = process.stdout }) {
  const inputs = { workspace: env.GITHUB_WORKSPACE ?? cwd }
  for (const name of INPUTS) inputs[name] = env[`INPUT_${name.toUpperCase()}`] ?? ''

  const resolve = (relative) => path.join(cwd, relative)
  const stat = (relative) => {
    try {
      return kindOf(fs.statSync(resolve(relative)))
    } catch {
      return null
    }
  }
  const list = (dir) =>
    fs.readdirSync(resolve(dir), { withFileTypes: true }).map((entry) => [entry.name, kindOf(entry)])
  const read = (relative) => {
    try {
      return fs.readFileSync(resolve(relative))
    } catch {
      return null
    }
  }

  const outcome = JSON.parse(wasm.run_action(JSON.stringify(inputs), stat, list, read))
  out.write(outcome.stdout)
  if (outcome.summary_markdown && env.GITHUB_STEP_SUMMARY) {
    fs.appendFileSync(env.GITHUB_STEP_SUMMARY, outcome.summary_markdown)
  }
  const results = inputs['output-json'].trim()
  if (outcome.results_json !== null && results) {
    const file = path.resolve(cwd, results)
    fs.mkdirSync(path.dirname(file), { recursive: true })
    fs.writeFileSync(file, outcome.results_json)
  }
  if (env.GITHUB_OUTPUT) {
    fs.appendFileSync(
      env.GITHUB_OUTPUT,
      `findings=${outcome.findings}\nerrors=${outcome.errors}\nwarnings=${outcome.warnings}\n` +
        `results-file=${results ? path.resolve(cwd, results) : ''}\n`,
    )
  }
  return outcome
}
