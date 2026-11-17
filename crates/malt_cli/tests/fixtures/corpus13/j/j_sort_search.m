function [idx, cmp] = sort_search(v, target)
%SORT_SEARCH Binary search on a sorted copy of v, with a comparison count.

[sortedV, perm] = sort(v);
lo = 1;
hi = numel(sortedV);
cmp = 0;
idx = [];
while lo <= hi
    mid = floor((lo + hi) / 2);
    cmp = cmp + 1;
    if sortedV(mid) == target
        idx = perm(mid);
        break
    elseif sortedV(mid) < target
        lo = mid + 1;
    else
        hi = mid - 1;
    end
end
if isempty(idx)
    idx = 0;
end

function out = insertion_sort(x)
out = x;
for i = 2:numel(out)
    key = out(i);
    j = i - 1;
    while j >= 1 && out(j) > key
        out(j + 1) = out(j);
        j = j - 1;
    end
    out(j + 1) = key;
end

function m = median_of(x)
s = insertion_sort(x);
n = numel(s);
if mod(n, 2)
    m = s((n + 1) / 2);
else
    m = (s(n / 2) + s(n / 2 + 1)) / 2;
end
