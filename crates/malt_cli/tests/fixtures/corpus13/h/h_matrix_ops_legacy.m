function [A, info] = matrix_ops_legacy(n, density)
% MATRIX_OPS_LEGACY Random sparse matrix with a few diagnostics. Old style.

if nargin < 2
    density = 0.1;
end
rand('state', 1);
A = sprand(n, n, density) + speye(n);
info.nnz = nnz(A);
info.cond = condest(A);
d = diag(A);
info.minDiag = min(d);
info.maxDiag = max(d);
B = A';
C = A * B;
D = C(1:2:end, 1:2:end);
trace_D = trace(D);
e = eig(full(D(1:min(5, end), 1:min(5, end))));
emin = min(abs(e));

tic
for i = 1:n
    for j = 1:n
        if A(i, j) ~= 0
            count = count + 1;
        end
    end
end
t = toc;
fprintf('nnz %d, cond %.3g, took %.3g s\n', info.nnz, info.cond, t);
info.sum = full(sum(A(:)))
v = zeros(1, 0);
for k = 1:n
    v = [v k^2];
end
end
