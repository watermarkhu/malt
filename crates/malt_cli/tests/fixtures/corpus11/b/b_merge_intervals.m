function merged = merge_intervals(intervals)
%MERGE_INTERVALS Merges overlapping closed intervals given as an N-by-2 array.
%   merged = MERGE_INTERVALS([1 3; 2 6; 8 10]) returns [1 6; 8 10].

if isempty(intervals)
    merged = zeros(0, 2);
    return
end
if ~isrow(intervals(1, :)) && size(intervals, 2) ~= 2
    error('merge:shape', 'intervals must have two columns');
end

[~, order] = sort(intervals(:, 1));
sorted = intervals(order, :);
merged = sorted(1, :);
for k = 2:size(sorted, 1)
    last = merged(end, :);
    cur = sorted(k, :);
    if cur(1) <= last(2)
        merged(end, 2) = max(last(2), cur(2));
    else
        merged = [merged; cur];
    end
end

lens = merged(:, 2) - merged(:, 1);
labels = cellfun(@(r) sprintf('[%g,%g]', r(1), r(2)), num2cell(merged, 2), 'UniformOutput', false);
txt = cell2mat(labels)
pairs = cell2mat(cellfun(@(c) c', num2cell(merged, 2), 'UniformOutput', false));
flat = reshape(merged', 1, [])
total = sum(lens);
longest = max(lens);
n = numel(lens);
idx = find(lens == longest);
for j = 1:numel(idx)
    j = idx(j);
end
end
