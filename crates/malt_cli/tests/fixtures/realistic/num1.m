function [x, iters] = num1(A, b, tol, maxit)
%NUM1 Conjugate gradient solver.
if nargin < 3 || isempty(tol), tol = 1e-8; end
if nargin < 4 || isempty(maxit), maxit = 100; end
n = size(A, 1);
x = zeros(n, 1);
r = b - A * x;
p = r;
rs = r' * r;
for iters = 1:maxit
    Ap = A * p;
    alpha = rs / (p' * Ap);
    x = x + alpha * p;
    r = r - alpha * Ap;
    rsnew = r' * r;
    if sqrt(rsnew) < tol
        break
    end
    p = r + (rsnew / rs) * p;
    rs = rsnew;
end
end
