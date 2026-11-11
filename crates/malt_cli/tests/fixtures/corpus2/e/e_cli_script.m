%% e_cli_script: nightly batch launcher for the sensor pipeline
% Sets paths, prepares the output folder, runs the stages and keeps a diary.
% Run from the project root: >> e_cli_script

%% Setup
close all; clc
if exist('cfg', 'var') && isstruct(cfg)
    fprintf('Re-using config from workspace\n');
else
    cfg = struct();
end

projectRoot = fileparts(mfilename('fullpath'));
addpath(genpath(fullfile(projectRoot, 'src')));
addpath(fullfile(projectRoot, 'lib'));

rng(42)
startTime = datestr(now, 'yyyy-mm-dd HH:MM:SS');
fprintf('Batch started %s\n', startTime);

%% Output folders
outDir = fullfile(projectRoot, 'output', datestr(now, 'yyyymmdd'));
if ~exist(outDir, 'dir')
    mkdir(outDir);
end

logFile = fullfile(outDir, 'batch.log');
diary(logFile)
diary on

%% Copy the configuration used for this run
cfgSource = fullfile(projectRoot, 'config', 'sensors.json');
cfgCopy = fullfile(outDir, 'sensors.json');
if exist(cfgSource, 'file') == 2
    copyfile(cfgSource, cfgCopy);
else
    warning('Config file missing: %s', cfgSource);
end

%% Stage 1: load raw data
tic
rawFiles = dir(fullfile(projectRoot, 'raw', '*.csv'));
fprintf('Found %d raw files\n', numel(rawFiles));
data = cell(numel(rawFiles), 1);
for i = 1:numel(rawFiles)
    data{i} = readmatrix(fullfile(rawFiles(i).folder, rawFiles(i).name));
end
t1 = toc;
fprintf('Stage 1 took %.2f s\n', t1);

%% Stage 2: clean and combine
tic
clean = cellfun(@removeOutliers, data, 'UniformOutput', false);
combined = cell2mat(clean);
summaryStats = [mean(combined, 1); std(combined, 0, 1)];
t2 = toc;
fprintf('Stage 2 took %.2f s, %d rows\n', t2, size(combined, 1));

%% Stage 3: write results
resultFile = fullfile(outDir, 'summary.txt');
writeSummary(resultFile, summaryStats, startTime);

%% Wrap up
clearvars -except cfg projectRoot outDir
diary off
fprintf('Done. Results in %s\n', outDir);

%% Local functions
function clean = removeOutliers(x)
% Drop rows more than 4 standard deviations from the column mean.
mu = mean(x, 1, 'omitnan');
sigma = std(x, 0, 1, 'omitnan');
keep = all(abs(x - mu) <= 4 * sigma, 2);
clean = x(keep, :);
end

function writeSummary(fileName, stats, stamp)
fid = fopen(fileName, 'w');
if fid < 0
    error('Cannot open %s for writing', fileName);
end
cleanup = onCleanup(@() fclose(fid));
fprintf(fid, 'Generated %s\n', stamp);
for col = 1:size(stats, 2)
    fprintf(fid, 'col %d  mean=%.4f  std=%.4f\n', col, stats(1, col), stats(2, col));
end
end
