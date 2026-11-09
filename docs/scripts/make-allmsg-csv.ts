// Turn the output of MATLAB's `checkcode(file, '-id', '-allmsg')` into the CSV that
// check_against_mathworks.ts reads.
//
//   bun docs/scripts/make-allmsg-csv.ts <matlab-allmsg.txt> <out.csv>
//
// Produce the input from MATLAB (the `matlab` MCP server or a desktop session):
//
//   s = evalc("checkcode('any_file.m','-id','-allmsg')");
//   fid = fopen('matlab-allmsg.txt','w'); fwrite(fid,s); fclose(fid);
//
// The output is MathWorks content: keep both files outside the repository
// (~/.local/share/malt-reference/). `checkcode` prints one line per check,
// `ID  <level>  message`, under `CODE  ========== Category ==========` headers.
// Levels map to severities as 3, 2 -> Error, 1 -> Warning, 0 and 5 -> Info (5 is
// the cyclomatic-complexity metrics); this was derived from the checks whose
// severity had been verified against the index.
const [inp, out] = Bun.argv.slice(2)
if (!inp || !out) { console.error('usage: bun make-allmsg-csv.ts <matlab-allmsg.txt> <out.csv>'); process.exit(2) }

const severity: Record<string, string> = { '5': 'Info', '3': 'Error', '2': 'Error', '1': 'Warning', '0': 'Info' }
const q = (s: string) => '"' + s.replace(/"/g, '""') + '"'
const rows = ['category,check_id,severity,message,message_release,can_be_disabled,related_docs']
let category = ''
for (const line of (await Bun.file(inp).text()).split('\n')) {
  const head = line.match(/^\s*\S+\s+={5,}\s*(.*?)\s*={5,}\s*$/)
  if (head) { category = head[1]; continue }
  const m = line.match(/^\s*([A-Za-z][A-Za-z0-9_.]*)\s+(\d)\s{2,}(.*)$/)
  if (!m) continue
  const [, id, level, message] = m
  if (!severity[level]) throw new Error(`unknown level ${level} for ${id}: extend the mapping`)
  rows.push([category, id, severity[level], message.trim(), '', '', ''].map(q).join(','))
}
await Bun.write(out, rows.join('\n') + '\n')
console.log(`${rows.length - 1} checks written to ${out}`)
