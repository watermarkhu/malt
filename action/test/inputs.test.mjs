import { describe, expect, test } from 'bun:test'
import { fails, readInputs, shows } from '../src/inputs.mjs'

describe('readInputs', () => {
  test('defaults', () => {
    expect(readInputs({})).toEqual({
      paths: ['.'],
      preset: '',
      level: 'info',
      failOn: 'error',
      annotations: true,
      summary: true,
      outputJson: '',
    })
  })

  test('paths split on spaces and newlines', () => {
    expect(readInputs({ INPUT_PATHS: 'src  lib\n  test/unit\r\n' }).paths).toEqual(['src', 'lib', 'test/unit'])
  })

  test('reads every input, case-insensitively for the choices', () => {
    const inputs = readInputs({
      INPUT_PRESET: 'MonkeyProof',
      INPUT_LEVEL: 'Warning',
      'INPUT_FAIL-ON': 'never',
      INPUT_ANNOTATIONS: 'false',
      INPUT_SUMMARY: 'no',
      'INPUT_OUTPUT-JSON': 'malt.json',
    })
    expect(inputs).toMatchObject({
      preset: 'monkeyproof',
      level: 'warning',
      failOn: 'never',
      annotations: false,
      summary: false,
      outputJson: 'malt.json',
    })
  })

  test('bad values name the input', () => {
    expect(() => readInputs({ INPUT_PRESET: 'strict' })).toThrow(/'preset'.*strict/)
    expect(() => readInputs({ INPUT_LEVEL: 'debug' })).toThrow(/'level'/)
    expect(() => readInputs({ 'INPUT_FAIL-ON': 'always' })).toThrow(/'fail-on'/)
    expect(() => readInputs({ INPUT_SUMMARY: 'maybe' })).toThrow(/'summary'/)
  })
})

describe('severity helpers', () => {
  test('shows', () => {
    expect(shows('error', 'error')).toBe(true)
    expect(shows('error', 'warning')).toBe(false)
    expect(shows('warning', 'warning')).toBe(true)
    expect(shows('warning', 'info')).toBe(false)
    expect(shows('info', 'info')).toBe(true)
  })

  test('fails', () => {
    expect(fails('error', 'error')).toBe(true)
    expect(fails('error', 'warning')).toBe(false)
    expect(fails('warning', 'warning')).toBe(true)
    expect(fails('info', 'info')).toBe(true)
    expect(fails('never', 'error')).toBe(false)
  })
})
