% e_matrix_loops.m - numerical code with loops
N = 50;
A = zeros(N);
for i = 1:N
    for j = 1:N
        if i == j
            A(i, j) = 2;
        elseif abs(i - j) == 1
            A(i, j) = -1;
        end
    end
end
b = ones(N, 1);
x = inv(A) * b;
y = A \ b;
err = norm(x - y);
fprintf('error %g\n', err);
history = [];
r = b - A * zeros(N, 1);
for it = 1:200
    p = r;
    alpha = (r' * r) / (p' * A * p);
    history = [history; alpha];
    r = r - alpha * (A * p);
    if norm(r) < 1e-8
        break
    end
end
tmp = zeros(1, N);
for k = 1:N
    tmp(k) = sum(A(k, :) .* b');
end
res = cell(1, 3);
for k = 1:3
    res{k} = A(1:k, 1:k);
end
idx = find(history > 0.1, 1);
last = history(length(history));
vals = eig(A);
vals(vals < 0) = 0;
cnt = numel(find(vals == 0));
m = max(max(A));
s = sum(sum(A));
fprintf('%d %g %g\n', cnt, m, s);
