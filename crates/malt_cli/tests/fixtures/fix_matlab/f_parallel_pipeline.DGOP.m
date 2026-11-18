function summary = f_parallel_pipeline(fileList, outDir)
%F_PARALLEL_PIPELINE Process a folder of data files in parallel.
%   SUMMARY = F_PARALLEL_PIPELINE(FILELIST) computes per-file statistics
%   with parfor, streams checksums with parfeval, and reduces a global
%   total with spmd.
%
%   SUMMARY = F_PARALLEL_PIPELINE(FILELIST, OUTDIR) also sets where the
%   results are written.

if nargin < 2
    outDir = fullfile(pwd, 'results');
end
if ~isfolder(outDir)
    mkdir(outDir);
end

pool = gcp('nocreate');
if isempty(pool)
    pool = parpool('local', 4);
end

% Constant lookup table: each worker loads it once
calib = parallel.pool.Constant(@() load('calibration.mat'));

nFiles = numel(fileList);
stats = zeros(nFiles, 3);

% parfor over independent files
parfor k = 1:nFiles
    raw = readmatrix(fileList{k});
    cal = calib.Value;
    scaled = raw * cal.gain;
    stats(k, :) = [mean(scaled), std(scaled), max(scaled)];
end

% Stream checksums back in completion order
futures(nFiles) = parallel.FevalFuture;
for k = 1:nFiles
    futures(k) = parfeval(pool, @f_file_checksum, 1, fileList{k});
end

checksums = strings(nFiles, 1);
for k = 1:nFiles
    [idx, val] = fetchNext(futures);
    checksums(idx) = string(val);
    fprintf('Checksum %d/%d done\n', k, nFiles);
end

% spmd: each lab sums its share, gop does the global reduction
spmd
    localTotal = sum(stats(labindex:numlabs:end, 1));
    grandTotal = spmdReduce(@plus, localTotal);
end

summary = struct('stats', stats, 'total', grandTotal{1}, ...
    'checksums', checksums, 'outDir', outDir);

% One batch job for an overnight rerun, waited on with a timeout
job = batch(@f_file_checksum, 1, {fileList{1}});
wait(job, 'finished', 300);
if strcmp(job.State, 'finished')
    fprintf('Batch result: %s\n', job.fetchOutputs{1});
end

ds = datastore(fileList, 'Type', 'text');
fprintf('Datastore has %d files\n', numel(ds.Files));
end
