// The inputs of the action, read from the INPUT_* environment variables the runner sets.

const LEVELS = ['error', 'warning', 'info']
const FAIL_ON = ['error', 'warning', 'info', 'never']
const PRESETS = ['all', 'mathworks', 'recommended', 'monkeyproof']

/** The raw value of an input: the runner upper-cases the name and turns spaces into `_`. */
function raw(env, name) {
  return (env[`INPUT_${name.replace(/ /g, '_').toUpperCase()}`] ?? '').trim()
}

function oneOf(env, name, allowed, fallback) {
  const value = raw(env, name).toLowerCase() || fallback
  if (!allowed.includes(value)) {
    throw new Error(`Input '${name}' must be one of ${allowed.join(', ')}, not '${value}'`)
  }
  return value
}

function flag(env, name, fallback) {
  const value = raw(env, name).toLowerCase()
  if (value === '') return fallback
  if (['true', 'yes', 'on', '1'].includes(value)) return true
  if (['false', 'no', 'off', '0'].includes(value)) return false
  throw new Error(`Input '${name}' must be true or false, not '${value}'`)
}

/** The inputs of a run, validated, with their defaults. Throws a message for a bad value. */
export function readInputs(env = process.env) {
  const paths = raw(env, 'paths')
    .split(/[\r\n]+/)
    .flatMap((line) => line.trim().split(/\s+/))
    .filter(Boolean)
  return {
    paths: paths.length > 0 ? paths : ['.'],
    // An empty preset keeps the one of `.malt.toml`; a set one overrules it.
    preset: raw(env, 'preset') === '' ? '' : oneOf(env, 'preset', PRESETS, ''),
    level: oneOf(env, 'level', LEVELS, 'info'),
    failOn: oneOf(env, 'fail-on', FAIL_ON, 'error'),
    annotations: flag(env, 'annotations', true),
    summary: flag(env, 'summary', true),
    outputJson: raw(env, 'output-json'),
  }
}

/** The rank of a severity: the lower, the more severe. */
export const RANK = { error: 0, warning: 1, info: 2 }

/** Whether a finding of `severity` is shown at `level` (`--level` of the CLI). */
export function shows(level, severity) {
  return RANK[severity] <= RANK[level]
}

/** Whether a finding of `severity` fails the step with `failOn`. */
export function fails(failOn, severity) {
  return failOn !== 'never' && RANK[severity] <= RANK[failOn]
}
