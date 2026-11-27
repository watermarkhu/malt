// One run of the action: read the inputs and `.malt.toml`, find the files, lint them with
// the WebAssembly build, then write the annotations, the step summary, the JSON file and the
// outputs, and decide whether the step fails. The formats are written by the same Rust code
// as the CLI's (`malt_core::report`), reached through the wasm package.

import fs from 'node:fs'
import path from 'node:path'
import { discover } from './discover.mjs'
import { fails, readInputs, shows } from './inputs.mjs'

const CONFIG_FILE = '.malt.toml'

/** Decode a file as UTF-8, else as Latin-1, as the CLI does. */
function decode(buffer) {
  try {
    return new TextDecoder('utf-8', { fatal: true }).decode(buffer)
  } catch {
    return buffer.toString('latin1')
  }
}

/** Append `text` to the file the runner names in `variable`, if it names one. */
function appendTo(env, variable, text) {
  if (env[variable]) fs.appendFileSync(env[variable], text)
}

/** The workflow-command escaping of a message. */
function escapeData(text) {
  return text.replace(/%/g, '%25').replace(/\r/g, '%0D').replace(/\n/g, '%0A')
}

/**
 * Run the action. `wasm` is the loaded malt-wasm module, `env` the environment, `cwd` the
 * workspace and `out` where workflow commands and log lines are written.
 * Returns the outcome; `failed` says whether the step should fail.
 */
export function run({ wasm, env = process.env, cwd = process.cwd(), out = process.stdout }) {
  const inputs = readInputs(env)
  const log = (line) => out.write(line + '\n')

  const configPath = path.join(cwd, CONFIG_FILE)
  const hasConfig = fs.existsSync(configPath)
  const toml = hasConfig ? fs.readFileSync(configPath, 'utf8') : ''
  // The inputs overrule the file: an empty preset leaves the file's.
  const overrides = inputs.preset ? { preset: inputs.preset } : {}
  const linter = wasm.Linter.from_toml_with(toml, JSON.stringify(overrides))
  for (const warning of JSON.parse(linter.get_config_warnings())) {
    log(`::warning file=${CONFIG_FILE},title=malt configuration::${escapeData(warning)}`)
  }

  const settings = JSON.parse(linter.get_file_settings())
  const { files, missing, warnings } = discover(inputs.paths, settings, cwd)
  for (const warning of warnings) log(`::warning::${escapeData(warning)}`)

  let records = missing.map((file) => ({
    file,
    line: 1,
    column: 1,
    rule: 'NOFIL',
    severity: 'error',
    message: `Unable to open file ${file}. File is not found.`,
    fix: null,
  }))
  for (const file of files) {
    const source = decode(fs.readFileSync(path.join(cwd, file)))
    records.push(...JSON.parse(linter.check_records(source, file)))
  }
  records = records.filter((record) => shows(inputs.level, record.severity))
  const recordsJson = JSON.stringify(records)

  if (files.length === 0 && missing.length === 0) {
    log(`::warning::malt found no MATLAB files under ${inputs.paths.join(', ')}`)
  }
  if (inputs.annotations && records.length > 0) {
    log(wasm.github_annotations(recordsJson, env.GITHUB_WORKSPACE ?? cwd))
  }
  if (inputs.summary) {
    appendTo(env, 'GITHUB_STEP_SUMMARY', wasm.step_summary(recordsJson, files.length))
  }
  const filesJson = JSON.stringify([...missing, ...files])
  if (inputs.outputJson) {
    const results = linter.results_json(recordsJson, filesJson, hasConfig ? CONFIG_FILE : undefined)
    fs.writeFileSync(path.resolve(cwd, inputs.outputJson), results + '\n')
  }

  const count = (severity) => records.filter((r) => r.severity === severity).length
  const counts = { findings: records.length, errors: count('error'), warnings: count('warning'), infos: count('info') }
  appendTo(
    env,
    'GITHUB_OUTPUT',
    [
      `findings=${counts.findings}`,
      `errors=${counts.errors}`,
      `warnings=${counts.warnings}`,
      `results-file=${inputs.outputJson ? path.resolve(cwd, inputs.outputJson) : ''}`,
      '',
    ].join('\n'),
  )

  const failing = records.filter((record) => fails(inputs.failOn, record.severity)).length
  log(
    `malt ${wasm.Linter.get_version()}: linted ${files.length} file${files.length === 1 ? '' : 's'}, ` +
      `${counts.findings} finding${counts.findings === 1 ? '' : 's'} ` +
      `(${counts.errors} error, ${counts.warnings} warning, ${counts.infos} info)`,
  )
  if (failing > 0) {
    log(`::error title=malt::${failing} finding${failing === 1 ? '' : 's'} at or above '${inputs.failOn}'`)
  }
  return { ...counts, files: files.length, failed: failing > 0 }
}
