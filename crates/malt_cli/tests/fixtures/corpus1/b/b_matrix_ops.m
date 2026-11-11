function out = b_matrix_ops(A, b, opts)
%B_MATRIX_OPS Assorted linear-algebra helpers used by the solver package.
%   OUT = B_MATRIX_OPS(A, B) returns a struct with the solution of A*x = b,
%   the condition number, a sparsity figure and the cumulative row sums.
%
%   OUT = B_MATRIX_OPS(A, B, OPTS) with OPTS.tolZero (default 1e-12)
%   controls which entries count as zero.

if nargin < 3 || ~isfield(opts, 'tolZero')
    opts.tolZero = 1e-12;
end

if isempty(A) || isempty(b)
    error('b_matrix_ops:emptyInput', 'A and b must not be empty.');
end

[m, n] = size(A);
if size(b, 1) ~= m
    error('b_matrix_ops:sizeMismatch', ...
        'A has %d rows but b has %d.', m, size(b, 1));
end

% Solve the system; use least squares when A is not square
if m == n
    x = A \ b;
else
    x = pinv(A) * b;
end

% Mask of numerically zero entries and the fraction of non-zeros
zeroMask    = abs(A) < opts.tolZero;
nnzFraction = nnz(~zeroMask) / numel(A);

% Small Kronecker block built from the leading corner of A
I2 = eye(2);
K  = kron(I2, A(1:min(2, m), 1:min(2, n)));

% Cumulative sums along the rows, and the largest solution entries
rowCum = cumsum(A, 2);
[xSorted, order] = sort(x, 'descend');
topIdx = order(1:min(3, numel(order)));

% Residual and condition number
res     = A * x - b;
resNorm = norm(res) / max(norm(b), eps);
condA   = cond(A);

% Rows that are all zero, and columns that contain a NaN
emptyRows = find(all(zeroMask, 2));
badCols   = find(any(isnan(A), 1));

% Rows whose residual is already small, collected in a growing list
goodRows = [];
for i = 1:m
    if abs(res(i)) < 1e-6
        goodRows(end+1) = i; %#ok<AGROW>
    end
end

% Rank with a tolerance scaled to the matrix norm
rk = rank(A, opts.tolZero * norm(A));

% Normal-equation pseudo-inverse, written with explicit transposes
if m >= n && rk == n
    Aplus = (A.' * A) \ A.';
else
    Aplus = [];
end

% Logical masks on the solution
positive = x(x > 0);
if isempty(positive)
    lastPositive = NaN;
else
    lastPositive = positive(end);
end

allFinite = all(isfinite(x(:)));
anyNeg    = any(x(:) < 0);

% Pack the results
out = struct();
out.x            = x;
out.topIdx       = topIdx;
out.topValues    = xSorted(1:numel(topIdx));
out.nnzFraction  = nnzFraction;
out.K            = K;
out.rowCum       = rowCum;
out.resNorm      = resNorm;
out.condA        = condA;
out.emptyRows    = emptyRows;
out.badCols      = badCols;
out.goodRows     = goodRows;
out.rank         = rk;
out.Aplus        = Aplus;
out.lastPositive = lastPositive;
out.allFinite    = allFinite;
out.anyNeg       = anyNeg;

if out.resNorm > 1e-3
    warning('b_matrix_ops:poorFit', 'Relative residual is %.3g.', out.resNorm);
end

end
