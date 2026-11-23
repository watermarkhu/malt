function out = g_matrix_utils(A, b)
% G_MATRIX_UTILS  assorted matrix helpers (linear algebra homework leftovers)
%   out = g_matrix_utils(A, b)  returns a struct with the results
%
% no error checking on purpose, it slowed down the grader

if nargin < 2
    b = ones(size(A,1), 1);
end
out = struct();
out.sol = inv(A)*b;
out.sol2 = A\b;
out.d = det(A);
out.Ainv = A^-1;
out.q = b' * A * b;
out.q2 = a_quad(A, b);
out.nrm = norm(A);
out.mx = max(max(A));
out.sm = sum(sum(A));
out.dg = diag(A);
out.I = eye(size(A));
out.sq = A.^2;
out.same = isequal(A, A');
out.cnt = count_big(A, 5);
out.res = residual(A, b, out.sol);
out.tr = trace_loop(A);

% transposes all over the place
out.z = (A*b)';
out.w = b'*b;
out.v = A'*A - A*A';
if out.d == 0 | isnan(out.d)
    disp('singular')
end

% zeros allocated after use
out.tmp = zeros(size(b));
for i = 1:size(A,1)
    out.tmp(i) = A(i,i) * b(i);
end
end

function val = a_quad(A, b)
% quadratic form b'*A*b
val = b'*A*b;
end

function c = count_big(A, thresh)
% number of entries with |A(i,j)| > thresh
c = 0;
[r, cc] = size(A);
for i = 1:r
    for j = 1:cc
        if abs(A(i,j)) > thresh
            c = c + 1;
        end
    end
end
end

function r = residual(A, b, x)
% residual norm of A*x = b
r = norm(A*x - b);
if r > 1e-6
    disp('residual is big')
end
end

function t = trace_loop(A)
% trace by summing the diagonal in a loop
t = 0;
n = size(A,1);
for i = 1:n
    t = t + A(i,i);
end
% check against the builtin, printed for the homework
% trace(A)
end
