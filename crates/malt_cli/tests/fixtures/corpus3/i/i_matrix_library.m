function out = i_matrix_library(A, b)
% I_MATRIX_LIBRARY  home-made linear algebra routines
%   out = i_matrix_library(A, b)

if nargin < 2
    b = ones(size(A, 1), 1);
end

out.matmul_ok = isequal(matmul(A, A), A*A);
out.T = transpose_loops(A);
out.x = gauss_solve(A, b);
[L, U, P] = lu_pivot(A);
out.lu_err = norm(P*A - L*U);
out.eig_dom = power_iteration(A, 100);
fprintf('residual %g\n', norm(A*out.x - b));
fprintf('lu error %g\n', out.lu_err);
end

function C = matmul(A, B)
% matrix product by three nested loops
[m, k] = size(A);
[k2, n] = size(B);
if k ~= k2
    error('dimension mismatch');
end
C = zeros(m, n);
for i = 1:m
    for j = 1:n
        s = 0;
        for p = 1:k
            s = s + A(i,p) * B(p,j);
        end
        C(i,j) = s;
    end
end
end

function T = transpose_loops(A)
% transpose element by element
[r, c] = size(A);
T = zeros(c, r);
for i = 1:r
    for j = 1:c
        T(j, i) = A(i, j);
    end
end
end

function x = gauss_solve(A, b)
% gaussian elimination with partial pivoting, then back substitution
n = size(A, 1);
M = [A b];
for k = 1:n-1
    [~, idx] = max(abs(M(k:n, k)));
    p = idx + k - 1;
    if p ~= k
        tmp = M(k, :);
        M(k, :) = M(p, :);
        M(p, :) = tmp;
    end
    if abs(M(k,k)) < 1e-10
        error('singular matrix');
    end
    for i = k+1:n
        f = M(i,k) / M(k,k);
        M(i,:) = M(i,:) - f * M(k,:);
    end
end
x = zeros(n, 1);
for i = n:-1:1
    s = M(i, n+1);
    for j = i+1:n
        s = s - M(i,j) * x(j);
    end
    x(i) = s / M(i,i);
end
end

function [L, U, P] = lu_pivot(A)
% LU decomposition with row pivoting
n = size(A, 1);
U = A;
L = eye(n);
P = eye(n);
for k = 1:n-1
    [~, idx] = max(abs(U(k:end, k)));
    p = k + idx - 1;
    if p ~= k
        U([k p], :) = U([p k], :);
        P([k p], :) = P([p k], :);
        L([k p], 1:k-1) = L([p k], 1:k-1);
    end
    for i = k+1:n
        L(i,k) = U(i,k) / U(k,k);
        U(i,:) = U(i,:) - L(i,k) * U(k,:);
    end
end
end

function lam = power_iteration(A, iters)
% dominant eigenvalue estimate
v = rand(size(A, 1), 1);
for it = 1:iters
    w = A * v;
    v = w / norm(w);
end
lam = v' * A * v;
end
