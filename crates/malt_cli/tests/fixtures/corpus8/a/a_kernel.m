function [Q, R, info] = a_kernel(A, tol)
%R4_KERNEL Householder QR with a rank estimate, written in the old style.
%   [Q, R, INFO] = R4_KERNEL(A, TOL) returns the factors and a struct INFO
%   with the numerical rank and the condition estimate.

if nargin < 2
    tol = 1e-12;
end

[m, n] = size(A);
Q = eye(m);
R = A;
for k = 1:min(m - 1, n)
    x = R(k:m, k);
    alpha = -sign(x(1)) * norm(x);
    v = x;
    v(1) = v(1) - alpha;
    if norm(v) < tol
        continue
    end
    v = v / norm(v);
    H = eye(m - k + 1) - 2 * (v * v');
    R(k:m, :) = H * R(k:m, :);
    Q(:, k:m) = Q(:, k:m) * H;
end

d = abs(diag(R));
rank_ = sum(d > tol * max(d));
cond_ = max(d) / max(min(d), eps);

[U, S, V] = svd(A);
s = diag(S);
e = eig(A' * A);
info.rank = rank_;
info.cond = cond_;
info.sv = s';
info.eigs = e

nSteps = 0;
history = [];
x0 = zeros(n, 1);
for it = 1:50
    x1 = x0 + 0.5 * (A' * (ones(m, 1) - A * x0));
    history(end + 1) = norm(x1 - x0); %#ok<AGROW>
    nSteps = nSteps + 1;
    if history(end) < tol
        break
    end
    x0 = x1;
end
info.steps = nSteps;
info.residual = norm(A * x0 - ones(m, 1));
trailing = U(:, end);
ratio = s(1) / s(end)
prodS = prod(s)
cumS = cumprod(s)
end
