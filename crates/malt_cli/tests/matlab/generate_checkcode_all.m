%GENERATE_CHECKCODE_ALL Run MATLAB's Code Analyzer with EVERY check turned on over the fixtures.
%   Phase 1 of the live check of the `--preset all` expectations (tests/matlab_conformance_all.rs
%   and tests/matlab_live_all.rs do phase 2). Same pattern as generate_checkcode.m; CI runs it with
%   matlab-actions/run-command:
%
%       run('crates/malt_cli/tests/matlab/generate_checkcode_all.m')
%
%   No arguments. Environment:
%     MALT_CHECKCODE_ALL_OUT   the TSV file to write
%                              (default: <repo>/target/matlab/checkcode_all.tsv)
%   Next to it, <stem>.release holds version('-release') (checkcode_all.release),
%   checkcode_all_messages.tsv the message text (MathWorks content: under target/ only, never
%   committed) and checkcode_all_files.txt every file that was checked.
%
%   The configuration. checkcode(file, '-id', '-config=<json>') takes a JSON file with
%   checks.<ID>.enabled, severity and limit (docs/VERIFY.md, 'MATLAB configuration file'). The
%   script writes one to a temporary folder (deleted at the end) that sets enabled=true for every
%   check ID the catalogue of checkcode('-allmsg', file) lists, so the checks that are off by
%   default report too. These are left out, because malt cannot be given the same setting or has
%   no such check (see docs/VERIFY.md, 'Preset all against MATLAB'):
%     - the naming checks (IDs with dots, naming.<entity>.<check>): MATLAB reports them only with
%       values, malt's defaults are other values; they are compared through
%       tests/matlab_config.rs, not here;
%     - the metrics CABE, SCABE, ACABE, MCABE, SMCABE, AMCABE (severity 5 information; malt has
%       no such check) and LLMCC (malt does not implement it);
%     - the configuration messages BDCFG, CFERR, BDOPT, CFIG and the custom-check example DEBUGMSG
%       (they describe the configuration file, not code);
%     - the disallow checks DAF* (malt's switches are off by default: both sides keep them off).
%   The threshold checks carry the limit of malt's CUSTOM_CHECKS defaults (the `all` preset),
%   so the two sides compare like with like: FCNIL 7, FCNOL 7, FCNLL 200, LLMNC 120, MNCSN 5,
%   CYCCOM and MCYCCOM 15, SCYCCOM and MSCYCCOM 20, ACYCCOM and MACYCCOM 10. SYSBANG is enabled
%   (malt's disallow_bang defaults to true).
%
%   The TSV has the columns of tests/fixtures/matlab_expected_all.tsv: path relative to the
%   fixtures folder (forward slashes), line (0 becomes 1) and check ID. A file that checkcode
%   fails on is written as the ID PROBE_ERROR, so it shows as a difference.

here = fileparts(mfilename('fullpath'));
repo = fileparts(fileparts(fileparts(fileparts(here))));
fixtures = fullfile(repo, 'crates', 'malt_cli', 'tests', 'fixtures');

out = getenv('MALT_CHECKCODE_ALL_OUT');
if isempty(out)
    out = fullfile(repo, 'target', 'matlab', 'checkcode_all.tsv');
end
if ~isfolder(fileparts(out))
    mkdir(fileparts(out));
end
[outFolder, outStem] = fileparts(out);
releaseFile = fullfile(outFolder, [outStem '.release']);

% Every .m file under the fixtures, at any depth: the same set the conformance test reads.
files = sort(listMfiles(fixtures, ''));

release = version('-release');
rf = fopen(releaseFile, 'w', 'n', 'UTF-8');
if rf < 0
    error('generate_checkcode_all:release', 'cannot write %s', releaseFile);
end
fprintf(rf, '%s\n', release);
fclose(rf);

% The configuration: every ID of the catalogue, minus the left-out ones.
work = tempname;
mkdir(work);
cleanWork = onCleanup(@() rmdir(work, 's'));
catalogue = fullfile(work, 'catalogue.m');
cf = fopen(catalogue, 'w');
fprintf(cf, 'x = 1;\n');
fclose(cf);
[ids, nNaming] = catalogueIds(catalogue);
skip = {'CABE', 'SCABE', 'ACABE', 'MCABE', 'SMCABE', 'AMCABE', 'LLMCC', ...
        'BDCFG', 'CFERR', 'BDOPT', 'CFIG', 'DEBUGMSG', ...
        'DAFTC', 'DAFPV', 'DAFCO', 'DAFBR', 'DAFRT', 'DAFSC', 'DAFNF', 'DAFCF', ...
        'DAFAF', 'DAFCV', 'DAFCVC', 'DAFVI', 'DAFVO'};
limits = struct('FCNIL', 7, 'FCNOL', 7, 'FCNLL', 200, 'LLMNC', 120, 'MNCSN', 5, ...
    'CYCCOM', 15, 'MCYCCOM', 15, 'SCYCCOM', 20, 'MSCYCCOM', 20, 'ACYCCOM', 10, 'MACYCCOM', 10);
checks = struct();
for k = 1:numel(ids)
    id = ids{k};
    if any(strcmp(id, skip))
        continue
    end
    setting = struct('enabled', true);
    if isfield(limits, id)
        setting.limit = limits.(id);
    end
    checks.(id) = setting;
end
configFile = fullfile(work, 'codeAnalyzerConfiguration.json');
jf = fopen(configFile, 'w', 'n', 'UTF-8');
fprintf(jf, '%s', jsonencode(struct('checks', checks)));
fclose(jf);
fprintf('configuration: %d checks enabled (%d catalogue IDs, %d naming IDs and %d others left out)\n', ...
    numel(fieldnames(checks)), numel(ids), nNaming, numel(skip));

fid = fopen(out, 'w', 'n', 'UTF-8');
if fid < 0
    error('generate_checkcode_all:open', 'cannot write %s', out);
end
cleaner = onCleanup(@() fclose(fid));
fprintf(fid, '# What MATLAB %s checkcode reports with every check enabled, for the fixture files: path, line, check ID.\n', release);
fprintf(fid, '# Generated by generate_checkcode_all.m (live check). Not committed.\n');

messagesFile = fullfile(outFolder, [outStem '_messages.tsv']);
fm = fopen(messagesFile, 'w', 'n', 'UTF-8');
if fm < 0
    error('generate_checkcode_all:open', 'cannot write %s', messagesFile);
end
cleanerM = onCleanup(@() fclose(fm));
fprintf(fm, '# What MATLAB %s checkcode says for each finding (every check enabled): path, line, check ID, message.\n', release);
fprintf(fm, '# Generated by generate_checkcode_all.m (live check). MathWorks content: not committed.\n');

filesFile = fullfile(outFolder, [outStem '_files.txt']);
ff = fopen(filesFile, 'w', 'n', 'UTF-8');
if ff < 0
    error('generate_checkcode_all:open', 'cannot write %s', filesFile);
end
cleanerF = onCleanup(@() fclose(ff));

findings = 0;
errors = 0;
for k = 1:numel(files)
    rel = files{k};
    fprintf(ff, '%s\n', rel);
    try
        info = checkcode(fullfile(fixtures, strrep(rel, '/', filesep)), '-id', '-struct', ...
            ['-config=' configFile]);
    catch err
        errors = errors + 1;
        fprintf('checkcode failed on %s: %s\n', rel, strrep(err.message, newline, ' '));
        fprintf(fid, '%s\t1\tPROBE_ERROR\n', rel);
        continue
    end
    for m = 1:numel(info)
        line = max(1, info(m).line(1));
        fprintf(fid, '%s\t%d\t%s\n', rel, line, info(m).id);
        fprintf(fm, '%s\t%d\t%s\t%s\n', rel, line, info(m).id, oneLine(info(m).message));
        findings = findings + 1;
    end
end
fprintf('checkcode_all: %d files, %d findings, %d error(s); MATLAB %s; written to %s\n', ...
    numel(files), findings, errors, release, out);

function [ids, nNaming] = catalogueIds(file)
%CATALOGUEIDS The check IDs of the message catalogue (checkcode -allmsg), without the naming ones.
%   Each catalogue line is "<ID> <severity digit> <message>"; the naming IDs have dots.
info = checkcode('-allmsg', file);
lines = splitlines(strjoin({info.message}, newline));
tok = regexp(lines, '^\s*([A-Za-z][A-Za-z0-9_.]*)\s+\d+\s+\S', 'tokens', 'once');
names = {};
for k = 1:numel(tok)
    if ~isempty(tok{k})
        names{end+1} = tok{k}{1}; %#ok<AGROW>
    end
end
names = unique(names, 'stable');
isNaming = contains(names, '.');
nNaming = sum(isNaming);
ids = names(~isNaming);
end

function files = listMfiles(folder, rel)
%LISTMFILES The .m files under folder, as paths relative to it (forward slashes).
files = {};
entries = dir(folder);
for k = 1:numel(entries)
    name = entries(k).name;
    if any(strcmp(name, {'.', '..'}))
        continue
    end
    if isempty(rel)
        child = name;
    else
        child = [rel '/' name];
    end
    if entries(k).isdir
        files = [files, listMfiles(fullfile(folder, name), child)]; %#ok<AGROW>
    else
        [~, ~, ext] = fileparts(name);
        if strcmp(ext, '.m')
            files{end+1} = child; %#ok<AGROW>
        end
    end
end
end

function text = oneLine(text)
%ONELINE A message on one line: tabs and line breaks become spaces.
text = strrep(strrep(strrep(text, sprintf('\r\n'), ' '), newline, ' '), sprintf('\t'), ' ');
end
