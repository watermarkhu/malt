// Finding the files of a run, as the CLI does: a named file is linted as it is, a named
// folder is searched for the files `include` picks up, minus what `exclude` skips and what
// the `.gitignore` files of the tree ignore. The settings come from the resolved
// configuration (`Linter.get_file_settings`), so the CLI and the action read one source.

import fs from 'node:fs'
import path from 'node:path'
import ignore from 'ignore'

/** A path with forward slashes and no leading `./`. */
function posix(p) {
  return p.split(path.sep).join('/').replace(/^\.\//, '')
}

/** Find the files for `paths` (relative to `cwd`) under `settings`. */
export function discover(paths, settings, cwd) {
  const exclude = ignore().add(settings.exclude)
  const include = ignore().add(settings.include)
  const files = []
  const seen = new Set()
  const missing = []
  const warnings = []
  const add = (file) => {
    if (!seen.has(file)) {
      seen.add(file)
      files.push(file)
    }
  }

  /** Whether `rel` (relative to `cwd`) or a folder above it matches an exclude pattern. */
  const excluded = (rel, isDir) => {
    const parts = rel.split('/')
    for (let i = 1; i < parts.length; i++) {
      if (exclude.ignores(parts.slice(0, i).join('/') + '/')) return true
    }
    return exclude.ignores(isDir ? rel + '/' : rel)
  }

  const walk = (dir, rel, stack) => {
    let entries
    try {
      entries = fs.readdirSync(path.join(cwd, dir), { withFileTypes: true })
    } catch (error) {
      warnings.push(`skipped an unreadable folder: ${dir} (${error.code ?? error.message})`)
      return
    }
    let local = stack
    if (settings.respect_gitignore) {
      const gitignore = path.join(cwd, dir, '.gitignore')
      if (fs.existsSync(gitignore)) {
        local = [...stack, { base: rel, matcher: ignore().add(fs.readFileSync(gitignore, 'utf8')) }]
      }
    }
    entries.sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0))
    for (const entry of entries) {
      if (entry.isSymbolicLink()) continue
      const isDir = entry.isDirectory()
      if (!isDir && !entry.isFile()) continue
      const child = rel === '' ? entry.name : `${rel}/${entry.name}`
      if (excludedEntry(child, isDir)) continue
      if (local.some(({ base, matcher }) => matcher.ignores(relativeTo(base, child) + (isDir ? '/' : '')))) {
        continue
      }
      if (isDir) walk(path.join(dir, entry.name), child, local)
      else if (include.ignores(child)) add(child)
    }
  }

  // Only the entry itself is matched in the walk: an excluded folder is not entered. When the
  // folder that was named is itself excluded, naming it is asking for it, and what is below it
  // is matched relative to it (the `ignore` package would otherwise exclude it all through
  // its parent).
  let rebase = null
  const excludedEntry = (child, isDir) => {
    const tested = rebase === null ? child : child.slice(rebase.length + 1)
    return exclude.ignores(isDir ? tested + '/' : tested)
  }

  for (const named of paths) {
    const rel = posix(path.normalize(named)).replace(/\/$/, '') || '.'
    const absolute = path.join(cwd, rel)
    let stat = null
    try {
      stat = fs.statSync(absolute)
    } catch {
      stat = null
    }
    if (stat?.isDirectory()) {
      rebase = rel !== '.' && excluded(rel, true) ? rel : null
      walk(rel === '.' ? '.' : rel, rel === '.' ? '' : rel, [])
    } else if (stat === null) {
      missing.push(rel)
    } else if (settings.force_exclude && excluded(rel, false)) {
      continue
    } else {
      add(rel)
    }
  }
  return { files, missing, warnings }
}

/** `child` relative to the folder `base` (both relative to the working directory). */
function relativeTo(base, child) {
  return base === '' ? child : child.slice(base.length + 1)
}
