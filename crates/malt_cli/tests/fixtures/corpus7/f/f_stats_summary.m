function [stats, outliers] = f_stats_summary(data, groups)
%R3_STATS_SUMMARY Per-group summary statistics and outlier flags.
%   [STATS, OUTLIERS] = R3_STATS_SUMMARY(DATA, GROUPS) returns a struct
%   with count, mean, median and spread for each unique group label.

if nargin < 2
    groups = ones(size(data));
end
if ~isequal(size(data), size(groups))
    error('r3:stats:size', 'data and groups must match in size.');
end

[labels, ~, idx] = unique(groups, 'stable');
nGroups = numel(labels);
stats = repmat(struct('label', 0, 'n', 0, 'mean', NaN, 'median', NaN, 'sd', NaN), 1, nGroups);

for g = 1:nGroups
    vals = data(idx == g);
    stats(g).label = labels(g);
    stats(g).n = numel(vals);
    stats(g).mean = mean(vals, 'omitnan');
    stats(g).median = median(vals, 'omitnan');
    stats(g).sd = std(vals, 'omitnan');
end

counts = accumarray(idx, 1)';
sums = accumarray(idx, data(:))';
cumulative = cumsum(sums);
gaps = diff(sums);

outliers = false(size(data));
for g = 1:nGroups
    m = stats(g).mean;
    s = stats(g).sd;
    outliers(idx == g) = abs(data(idx == g) - m) > 3 * s;
end

hist = histcounts(data, 5);
[sorted, order] = sort(data, 'descend');
top3 = sorted(1:min(3, end));
firstBig = find(data > 100, 2);
anyNeg = any(data < 0);
allPos = all(data(:) > 0, 1)
flat = data(:)';
ratio = max(counts) / min(counts);
end
