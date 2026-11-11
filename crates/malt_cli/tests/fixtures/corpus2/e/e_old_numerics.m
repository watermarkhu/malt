function [peakIdx, nHits, table] = e_old_numerics(x, threshold)
% E_OLD_NUMERICS  Peak finding and counting, written the way we did it in 2004.
%   [peakIdx, nHits, table] = e_old_numerics(x, threshold)
%   peakIdx : index of the global maximum
%   nHits   : number of samples above threshold
%   table   : cell array with one row per sample above threshold

if nargin < 2
    threshold = 0.5;
end

x = x(:)';
n = length(x);

% find the max, the old way
m = max(x);
peakIdx = find(x == m);
if isempty(peakIdx)
    peakIdx = 0;
end

% count hits with an explicit loop
nHits = 0;
table = {};
for i = 1:length(x)
    if x(i) > threshold
        nHits = nHits + 1;
        table{end+1, 1} = i;
        table{end, 2} = x(i);
    end
end

% build a growing row vector
growth = zeros(1, 0);
for i = 1:n
    if mod(i, 3) == 0
        growth = [growth x(i)];
    end
end

% check for any NaN, a belt and braces way
hasBad = 0;
if size(x, 1) * size(x, 2) > 0
    hasBad = any(any(isnan(x)));
end
if hasBad
    fprintf('Warning: NaN found in input\n');
end

% print a summary line for each hit
for k = 1:size(table, 1)
    s = sprintf('%d', table{k, 1});
    disp(['hit at sample ' s ' value ' num2str(table{k, 2})]);
end

% label each element with a string
labels = cell(1, n);
for i = 1:n
    labels{i} = num2str(i);
end

% match names against a list
names = {'alpha', 'beta', 'gamma'};
for i = 1:length(names)
    if strcmp(names{i}, 'beta')
        fprintf('beta is at position %d\n', i);
    end
end

% check that a value exists in a vector
if isempty(find(x == threshold, 1))
    fprintf('threshold %g not hit exactly\n', threshold);
end

% growth of a column vector list
results = [];
for i = 1:length(growth)
    results(end+1) = growth(i) * 2;
end
fprintf('%d doubled values\n', length(results));
end
