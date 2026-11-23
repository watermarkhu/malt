function out = i_sorting_algos(data, method)
% I_SORTING_ALGOS  sort a vector with one of several algorithms
%   out = i_sorting_algos(data, method)
%   method is 'bubble', 'insertion', 'merge' or 'quick'

if nargin < 2
    method = 'bubble';
end

if ~isnumeric(data)
    error('i_sorting_algos:input', 'data must be numeric');
end

switch method
    case 'bubble'
        out = bubble_sort(data);
    case 'insertion'
        out = insertion_sort(data);
    case 'merge'
        out = merge_sort(data);
    case 'quick'
        out = quick_sort(data);
    otherwise
        error('unknown method %s', method);
end

end

function a = bubble_sort(a)
% classic bubble sort, swaps with a temp variable
n = length(a);
swapped = true;
while swapped
    swapped = false;
    for i = 1:n-1
        if a(i) > a(i+1)
            tmp = a(i);
            a(i) = a(i+1);
            a(i+1) = tmp;
            swapped = true;
        end
    end
    n = n - 1;
end
end

function a = insertion_sort(a)
% insertion sort, shifting elements one slot at a time
for i = 2:numel(a)
    key = a(i);
    j = i - 1;
    while j >= 1 && a(j) > key
        a(j+1) = a(j);
        j = j - 1;
    end
    a(j+1) = key;
end
end

function out = merge_sort(a)
% recursive merge sort, returns early for short inputs
if numel(a) <= 1
    out = a;
    return;
end
mid = floor(size(a, 2) / 2);
left = merge_sort(a(1:mid));
right = merge_sort(a(mid+1:end));
out = merge_lists(left, right);
end

function out = merge_lists(l, r)
% merge two sorted row vectors
out = zeros(1, length(l) + length(r));
i = 1; j = 1; k = 1;
while i <= length(l) && j <= length(r)
    if l(i) <= r(j)
        out(k) = l(i); i = i + 1;
    else
        out(k) = r(j); j = j + 1;
    end
    k = k + 1;
end
while i <= length(l)
    out(k) = l(i); i = i + 1; k = k + 1;
end
while j <= length(r)
    out(k) = r(j); j = j + 1; k = k + 1;
end
end

function a = quick_sort(a)
% quick sort using logical indexing to split around the pivot
if length(a) < 2
    return
end
pivot = a(end);
left = a(a < pivot);
mid = a(a == pivot);
right = a(a > pivot);
a = [quick_sort(left), mid, quick_sort(right)];
end
