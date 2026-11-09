%% Monte Carlo
rng(42);
N = 1e5;
pts = rand(N, 2);
inside = sum(pts(:,1).^2 + pts(:,2).^2 <= 1);
piEst = 4 * inside / N;
fprintf('pi ~ %.4f\n', piEst);

M = magic(5);
M(M > 20) = 0;
col = M(:, 2);
rowmax = max(M, [], 2);
idx = find(rowmax == max(rowmax));
tbl = table(rowmax, 'VariableNames', {'m'});
tbl.m2 = tbl.m .^ 2;
histogram(col);
xlim([0 25]);
mask = ~isnan(col) & col > 0;
avg = mean(col(mask));
w = linspace(0, 1, 11)';
v = w .* w;
total = trapz(w, v);
B = reshape(1:6, 2, 3);
B(:, end+1) = [7; 8];
disp(size(B));
isequal(B, B');
