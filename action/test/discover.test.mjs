import { afterEach, describe, expect, test } from 'bun:test'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { discover } from '../src/discover.mjs'

const DEFAULTS = {
  include: ['*.m'],
  exclude: ['.git/', '.hg/', '.svn/', 'node_modules/', '.venv/', 'slprj/'],
  respect_gitignore: true,
  force_exclude: false,
}

const dirs = []
function project(files) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'malt-action-'))
  dirs.push(dir)
  for (const file of files) {
    fs.mkdirSync(path.dirname(path.join(dir, file)), { recursive: true })
    fs.writeFileSync(path.join(dir, file), 'x = 1;\n')
  }
  return dir
}
afterEach(() => {
  for (const dir of dirs.splice(0)) fs.rmSync(dir, { recursive: true, force: true })
})

const found = (dir, paths, settings = {}) => discover(paths, { ...DEFAULTS, ...settings }, dir)

describe('discover', () => {
  test('a folder gives its .m files, below it too, in path order', () => {
    const dir = project(['b.m', 'a.m', 'sub/c.m', 'notes.txt', 'data.mat'])
    expect(found(dir, ['.']).files).toEqual(['a.m', 'b.m', 'sub/c.m'])
  })

  test('a file is linted whatever its name, and once', () => {
    const dir = project(['a.txt', 'b.m'])
    expect(found(dir, ['a.txt', 'b.m', './b.m']).files).toEqual(['a.txt', 'b.m'])
  })

  test('a missing path is reported, not linted', () => {
    const dir = project(['a.m'])
    const result = found(dir, ['nope.m', 'a.m'])
    expect(result.files).toEqual(['a.m'])
    expect(result.missing).toEqual(['nope.m'])
  })

  test('the default excludes skip version control and dependencies', () => {
    const dir = project(['a.m', '.git/h.m', 'node_modules/p/y.m', 'slprj/z.m'])
    expect(found(dir, ['.']).files).toEqual(['a.m'])
  })

  test('exclude follows gitignore rules', () => {
    const dir = project(['a.m', 'test/t.m', 'test/deep/d.m', 'src/test/s.m', 'x_old.m'])
    expect(found(dir, ['.'], { exclude: ['/test/', '*_old.m'] }).files).toEqual(['a.m', 'src/test/s.m'])
  })

  test('include picks other files', () => {
    const dir = project(['a.m', 'b.txt', 'sub/c.txt'])
    expect(found(dir, ['.'], { include: ['*.txt'] }).files).toEqual(['b.txt', 'sub/c.txt'])
  })

  test('.gitignore files are respected, nested ones for their own folder, unless switched off', () => {
    const dir = project(['a.m', 'build/b.m', 'c.m', 'sub/d.m', 'sub/e.m'])
    fs.writeFileSync(path.join(dir, '.gitignore'), 'build/\nc.m\n')
    fs.writeFileSync(path.join(dir, 'sub', '.gitignore'), 'd.m\n')
    expect(found(dir, ['.']).files).toEqual(['a.m', 'sub/e.m'])
    expect(found(dir, ['.'], { respect_gitignore: false }).files).toEqual(['a.m', 'build/b.m', 'c.m', 'sub/d.m', 'sub/e.m'])
  })

  test('force_exclude applies the patterns to named files', () => {
    const dir = project(['a.m', 'vendor/v.m'])
    const settings = { exclude: ['vendor/'] }
    expect(found(dir, ['a.m', 'vendor/v.m'], settings).files).toEqual(['a.m', 'vendor/v.m'])
    expect(found(dir, ['a.m', 'vendor/v.m'], { ...settings, force_exclude: true }).files).toEqual(['a.m'])
  })

  test('a named folder below an excluded one is still searched', () => {
    const dir = project(['vendor/lib/l.m', 'vendor/other/o.m'])
    expect(found(dir, ['vendor/lib'], { exclude: ['vendor/'] }).files).toEqual(['vendor/lib/l.m'])
  })

  test('symbolic links are not followed', () => {
    const dir = project(['a.m', 'real/r.m'])
    fs.symlinkSync(path.join(dir, 'real'), path.join(dir, 'link'))
    expect(found(dir, ['.']).files).toEqual(['a.m', 'real/r.m'])
  })

  test('an empty folder finds nothing', () => {
    const dir = project([])
    expect(found(dir, ['.'])).toEqual({ files: [], missing: [], warnings: [] })
  })
})
