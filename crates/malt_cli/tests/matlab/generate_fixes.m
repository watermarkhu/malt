%GENERATE_FIXES Apply MATLAB's own fix to the committed fix_matlab pairs.
%   Phase 1 of the MATLAB-fix differential (tests/fix_matlab.rs does phase 2).
%   The committed expectation set is tests/fixtures/fix_matlab/<name>.<ID>.m:
%   <name> is the file name (without .m) of a fixture under tests/fixtures,
%   <ID> a check ID that MATLAB marks auto-fixable on it. For every such pair
%   this script copies the fixture under its own name into a work folder, runs
%   codeIssues (factory configuration) and issues.fix(issues, ID), and writes
%   the fixed text to <out>/<name>.<ID>.m plus one row per attempt to
%   <out>/fixes.tsv: expectation name, changed, success, error message.
%
%   The repository files are never written to; only the copies are fixed.
%   Run from the repository root, with MATLAB on PATH (as CI does):
%
%       run('crates/malt_cli/tests/matlab/generate_fixes.m')
%
%   Environment:
%     MALT_FIXES_OUT   the folder to write (default: <repo>/target/matlab/fixes)
%
%   The comparison test reads the same folder (MALT_MATLAB_FIXES) when MATLAB
%   itself must not be started, or runs this very script in a temporary folder
%   when it may.

here = fileparts(mfilename('fullpath'));
repo = fileparts(fileparts(fileparts(fileparts(here))));
fixtures = fullfile(repo, 'crates', 'malt_cli', 'tests', 'fixtures');
pairsRoot = fullfile(fixtures, 'fix_matlab');

out = getenv('MALT_FIXES_OUT');
if isempty(out)
    out = fullfile(repo, 'target', 'matlab', 'fixes');
end
if ~isfolder(out)
    mkdir(out);
end
work = fullfile(out, 'work');
if ~isfolder(work)
    mkdir(work);
end

list = dir(fullfile(pairsRoot, '*.m'));
if isempty(list)
    error('generate_fixes:empty', 'no expectation files under %s', pairsRoot);
end

fid = fopen(fullfile(out, 'fixes.tsv'), 'w', 'n', 'UTF-8');
if fid < 0
    error('generate_fixes:open', 'cannot write %s', fullfile(out, 'fixes.tsv'));
end
cleaner = onCleanup(@() fclose(fid));
fprintf(fid, '# MATLAB fix() output per committed pair: name, changed, success, error\n');

fixedCount = 0;
for k = 1:numel(list)
    [~, stemAndId, ~] = fileparts(list(k).name);
    dot = strfind(stemAndId, '.');
    if numel(dot) ~= 1
        fprintf(fid, '%s\t0\t0\tBAD_NAME\n', stemAndId);
        fprintf('skipped %s: the name must be exactly <fixture>.<ID>.m\n', stemAndId);
        continue
    end
    stem = stemAndId(1:dot(1) - 1);
    id = stemAndId(dot(1) + 1:end);

    sources = dir(fullfile(fixtures, '**', [stem '.m']));
    if isempty(sources)
        fprintf(fid, '%s\t0\t0\tNO_FIXTURE\n', stemAndId);
        fprintf('no fixture named %s.m under %s\n', stem, fixtures);
        continue
    end
    source = fullfile(sources(1).folder, sources(1).name);

    target = fullfile(work, [stem '.m']);
    copyfile(source, target);
    before = fileread(target);
    [changed, success, errorMessage] = deal(false, false, '');
    try
        issues = codeIssues(target, CodeAnalyzerConfiguration="factory");
        issues.fix(id);
        success = true;
        after = fileread(target);
        changed = ~strcmp(before, after);
        if changed
            fout = fopen(fullfile(out, [stemAndId '.m']), 'w', 'n', 'UTF-8');
            if fout < 0
                error('generate_fixes:open', 'cannot write %s', stemAndId);
            end
            fprintf(fout, '%s', after);
            fclose(fout);
            fixedCount = fixedCount + 1;
        end
    catch err
        success = false;
        errorMessage = strrep(err.message, newline, ' ');
    end
    fprintf(fid, '%s\t%d\t%d\t%s\n', stemAndId, changed, success, ...
        strrep(errorMessage, sprintf('\t'), ' '));
    fprintf('pair %d/%d: %s changed=%d success=%d %s\n', k, numel(list), stemAndId, ...
        changed, success, errorMessage);
end
fprintf('generate_fixes: %d of %d pair(s) fixed; written to %s\n', fixedCount, ...
    numel(list), out);
