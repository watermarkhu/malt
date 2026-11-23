function out = i_dynamic_programming(a, b)
% I_DYNAMIC_PROGRAMMING  knapsack, edit distance and LCS demos
%   out = i_dynamic_programming(a, b)

if nargin < 2
    a = 'kitten';
    b = 'sitting';
end

out.knap = knapsack([60 100 120], [10 20 30], 50);
out.edit = edit_distance(a, b);
out.lcs = lcs_length(a, b);
out.lcs_str = lcs_string(a, b);
fprintf('knapsack=%d edit=%d lcs=%d\n', out.knap, out.edit, out.lcs);
print_table(lcs_table(a, b), a, b);
end

function best = knapsack(values, weights, cap)
% 0/1 knapsack with a 2-D table, rows are items, columns are capacities
n = length(values);
dp = zeros(n+1, cap+1);
for i = 1:n
    for w = 0:cap
        dp(i+1, w+1) = dp(i, w+1);
        if weights(i) <= w
            cand = dp(i, w+1-weights(i)) + values(i);
            dp(i+1, w+1) = max(dp(i+1, w+1), cand);
        end
    end
end
best = dp(n+1, cap+1);
end

function d = edit_distance(a, b)
% levenshtein distance with 1-based index shifts
m = length(a);
n = length(b);
D = zeros(m+1, n+1);
for i = 0:m
    D(i+1, 1) = i;
end
for j = 0:n
    D(1, j+1) = j;
end
for i = 1:m
    for j = 1:n
        if a(i) == b(j)
            cost = 0;
        else
            cost = 1;
        end
        D(i+1, j+1) = min([D(i, j+1) + 1, D(i+1, j) + 1, D(i, j) + cost]);
    end
end
d = D(m+1, n+1);
end

function L = lcs_table(a, b)
% longest common subsequence table
m = numel(a); n = numel(b);
L = zeros(m+1, n+1);
for i = 1:m
    for j = 1:n
        if strcmp(a(i), b(j))
            L(i+1, j+1) = L(i, j) + 1;
        else
            L(i+1, j+1) = max(L(i, j+1), L(i+1, j));
        end
    end
end
end

function len = lcs_length(a, b)
% just the length, read from the corner of the table
L = lcs_table(a, b);
len = L(end, end);
end

function s = lcs_string(a, b)
% walk back through the table to rebuild one LCS
L = lcs_table(a, b);
i = length(a); j = length(b);
s = '';
while i > 0 && j > 0
    if a(i) == b(j)
        s = [a(i) s];
        i = i - 1; j = j - 1;
    elseif L(i, j+1) >= L(i+1, j)
        i = i - 1;
    else
        j = j - 1;
    end
end
end

function print_table(T, a, b)
% show a DP table with the strings as headers
fprintf('    ');
for j = 1:length(b)
    fprintf('%4s', b(j));
end
fprintf('\n');
for i = 1:size(T, 1)
    if i == 1
        fprintf('  ');
    else
        fprintf('%s ', a(i-1));
    end
    for j = 1:size(T, 2)
        fprintf('%4d', T(i, j));
    end
    fprintf('\n');
end
end
