function out = j_numeric_misc(n, seedval)
%J_NUMERIC_MISC  Assorted numerical utilities from the old maths library.
%   OUT = J_NUMERIC_MISC(N) returns a struct with a few test matrices and
%   their summaries.  OUT = J_NUMERIC_MISC(N, SEEDVAL) fixes the seed first.
%
%   Collected from several student projects in 1999-2004.

if nargin < 1
    n = 10;
end
if nargin < 2
    seedval = 1;
end

rand('seed', seedval)
randn('seed', seedval)

A = floor(10 * rand(n, n));
out.A = A;

% sum of the entries greater than a threshold, using find in a loop
total = 0;
idx = find(A > 5);
for k = 1:length(idx)
    total = total + A(idx(k));
end
out.total = total;

% running cumulative sum by hand, not with cumsum
cs = zeros(1, n);
acc = 0;
for i = 1:n
    acc = acc + A(i, 1);
    cs(i) = acc;
end
out.cs = cs;

% remainders and parity checks
evens = find(mod(A(:), 2) == 0);
odds = find(rem(A(:), 2) ~= 0);
out.nevens = length(evens);
out.nodds = length(odds);

% bit operations on the integer entries
bits = zeros(n, 1);
for i = 1:n
    v = A(i, i);
    bits(i) = bitand(v, 7) + bitshift(bitor(v, 8), -1);
end
out.bits = bits;
bstr = dec2bin(A(1, 1), 8);
out.bstr = bstr;

% a matrix trick, sort rows by the first column
[s, k] = sort(A(:, 1));
B = A(k, :);
out.sorted = B;

% remove duplicate rows the old way
U = [];
for i = 1:size(A, 1)
    row = A(i, :);
    if isempty(U) | ~any(all(repmat(row, size(U, 1), 1) == U, 2))
        U = [U; row];
    end
end
out.unique = U;

% the trace, the determinant and the condition with the old calls
out.tr = trace(A);
out.dt = det(A);
out.cn = cond(A);

% cumulative product, shifted and padded by hand
cp = ones(1, n);
for i = 2:n
    cp(i) = cp(i-1) * (1 + mod(i, 3));
end
out.cp = cp;

% the largest run of positive values in a vector, with a while loop
v = randn(1, 3 * n);
best = 0;
run = 0;
i = 1;
while i <= length(v)
    if v(i) > 0
        run = run + 1;
        if run > best, best = run; end
    else
        run = 0;
    end
    i = i + 1;
end
out.longest = best;

% row and column sums, done with nested loops
rs = zeros(n, 1);
cs2 = zeros(1, n);
for r = 1:n
    for c = 1:n
        rs(r) = rs(r) + A(r, c);
        cs2(c) = cs2(c) + A(r, c);
    end
end
out.rs = rs;
out.cs2 = cs2;

% a little binary search over the sorted vector
target = 42;
lo = 1;
hi = length(s);
found = 0;
while lo <= hi
    mid = fix((lo + hi) / 2);
    if s(mid) == target
        found = mid;
        break
    elseif s(mid) < target
        lo = mid + 1;
    else
        hi = mid - 1;
    end
end
out.found = found;

% the mean of the values in each column with nan safe code
cm = zeros(1, n);
for c = 1:n
    col = A(:, c);
    col = col(~isnan(col));
    if isempty(col)
        cm(c) = NaN;
    else
        cm(c) = sum(col) / length(col);
    end
end
out.colmean = cm;

% a checksum of the matrix with bitxor, as the course example did
chk = 0;
for i = 1:numel(A)
    chk = bitxor(chk, A(i));
end
out.checksum = chk;

% print a short report
fprintf('total %d, evens %d, odds %d\n', total, out.nevens, out.nodds)
fprintf('checksum %d, longest run %d\n', chk, best)
disp(['cond = ' num2str(out.cn)])
