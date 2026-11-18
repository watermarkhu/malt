function summary = d_batch_runner(inputDir, pattern, outputDir)
%D_BATCH_RUNNER Process every matching file in a folder, collecting errors.
%   SUMMARY = D_BATCH_RUNNER(INPUTDIR, PATTERN, OUTPUTDIR) calls
%   processFile on each file in INPUTDIR matching PATTERN (for example
%   '*.csv'). Failures are caught per file so one bad input does not stop
%   the batch. SUMMARY has fields nOK, nFailed, failures (N-by-2 cell array
%   of {file, message} rows) and elapsed (seconds).

if nargin < 2 || isempty(pattern)
    pattern = '*.csv';
end
if nargin < 3
    outputDir = fullfile(inputDir, 'out');
end

if ~exist(inputDir, 'dir')
    error('d_batch_runner:noFolder', 'Input folder does not exist: %s', inputDir);
end
if ~exist(outputDir, 'dir')
    mkdir(outputDir);
end

files = dir(fullfile(inputDir, pattern));
files = files(~[files.isdir]);
nFiles = numel(files);
fprintf('Found %d files matching %s\n', nFiles, pattern);

summary = struct('nOK', 0, 'nFailed', 0, 'failures', {cell(0, 2)}, 'elapsed', 0);
tStart = tic;

for k = 1:nFiles
    inPath = fullfile(files(k).folder, files(k).name);
    [~, base, ~] = fileparts(files(k).name);
    outPath = fullfile(outputDir, [base '_processed.mat']);

    try
        result = processFile(inPath);
        save(outPath, 'result');
        summary.nOK = summary.nOK + 1;
        status = 'ok';
    catch err
        summary.nFailed = summary.nFailed + 1;
        summary.failures(end+1, :) = {files(k).name, err.message};
        status = 'FAILED';
    end

    % Progress line in place of a waitbar
    fprintf('[%3d/%3d] %-30s %s\n', k, nFiles, files(k).name, status);
end

summary.elapsed = toc(tStart);
fprintf('Done: %d ok, %d failed, %.1f s\n', summary.nOK, summary.nFailed, summary.elapsed);
end

function result = processFile(path)
% Placeholder processing: load the table and report its size
tbl = readtable(path);
result = struct('rows', height(tbl), 'cols', width(tbl), 'source', path);
end
