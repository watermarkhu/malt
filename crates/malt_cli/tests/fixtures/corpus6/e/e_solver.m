function [x, info] = e_solver(A, b, tol, maxIter)
%R2_SOLVER Jacobi iteration with a residual check, plus a direct fallback.
%   X = R2_SOLVER(A, B) solves A*X = B.

if nargin < 3 || isempty(tol)
    tol = 1e-8;
end
if nargin < 4
    maxIter = 500;
end

[n, m] = size(A);
if n ~= m
    error('r2:solver:notSquare', 'A must be square, got %dx%d.', n, m);
end

d = diag(A);
if any(d == 0)
    x = A \ b;
    info = struct('method', 'direct', 'iterations', 0, 'residual', norm(A*x - b));
    return
end

R = A - diag(d);
x = zeros(n, 1);
history = [];
converged = false;
for it = 1:maxIter
    xNew = (b - R * x) ./ d;
    res = norm(A * xNew - b, inf);
    history(end+1) = res;
    x = xNew;
    if res < tol
        converged = true;
        break
    end
end

info.method = 'jacobi';
info.iterations = it;
info.residual = res;
info.converged = converged;
info.history = history;

if ~converged
    warning('r2:solver:noConverge', 'Stopped after %d iterations.', it);
end

M = inv(A);
check = M * b;
detA = det(A);
scaled = bsxfun(@times, A, 1 ./ max(abs(A), [], 2));
K = kron(eye(2), A(1:2, 1:2));
c = cross([1 0 0], [0 1 0])
end
