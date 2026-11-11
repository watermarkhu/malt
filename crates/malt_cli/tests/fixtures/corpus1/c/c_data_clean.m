%% Clean the raw sensor export before modelling
% Reads the raw CSV, removes NaNs and outliers, and stores the cleaned
% series together with some summary statistics.

clear; close all; clc

rawFile = 'sensor_export_2026.csv';
outFile = 'sensor_clean.mat';

T = readtable(rawFile);
x = T.reading;
t = T.timestamp;

fprintf('Loaded %d rows from %s\n', numel(x), rawFile)

%% Missing values
nMissing = sum(isnan(x))
if any(isnan(x))
    fprintf('Dropping %d NaN readings\n', nMissing);
end
meanRaw = mean(x, 'omitnan');
x(isnan(x)) = [];
t(isnan(x)) = [];

%% Outlier removal, 3 sigma around the median
med = median(x);
sigma = 1.4826 * median(abs(x - med));
isOutlier = abs(x - med) > 3 * sigma;
x = x(~isOutlier);
t = t(~isOutlier);

% Histogram of what is left
[counts, edges] = histcounts(x, 20);
cdf = cumsum(counts) / sum(counts);

%% Build the cleaned series, growing the array in a loop
cleaned = [];
prevValue = NaN;
for i = 1:numel(x)
    if isnan(prevValue) || abs(x(i) - prevValue) < 10
        cleaned(end+1) = x(i);
        prevValue = x(i);
    end
end

% Differences between consecutive samples
dx = diff(cleaned);
largeJumps = find(abs(dx) > 5);

%% Unique sensor IDs and per-sensor summary
sensorIds = unique(T.sensorId);
summary = struct();
for k = 1:numel(sensorIds)
    sid = sensorIds(k);
    sel = T.sensorId == sid;
    field = sprintf('sensor_%d', sid);
    summary.(field).count = nnz(sel);
    summary.(field).mean = mean(T.reading(sel), 'omitnan');
    summary.(field).maxValue = max(T.reading(sel));
end

%% Save results
meta.source = rawFile;
meta.createdAt = datestr(now);
meta.nMissing = nMissing;
meta.meanRaw = meanRaw;
meta.edges = edges;
meta.cdf = cdf;
save(outFile, 'cleaned', 'summary', 'meta', 'largeJumps');

% Reload to check the round trip
S = load(outFile);
if ~isequal(S.cleaned, cleaned)
    error('Round trip failed for %s', outFile);
end
fprintf('Saved %d cleaned samples to %s\n', numel(S.cleaned), outFile);
