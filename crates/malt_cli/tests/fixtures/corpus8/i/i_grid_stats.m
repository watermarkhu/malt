% R4_GRID_STATS  Grid statistics with logical masks, a histogram and a plot.
% Works on a generated grid so it runs without data.

[X, Y] = meshgrid(-2:0.1:2, -2:0.1:2);
Z = exp(-(X.^2 + Y.^2)) .* cos(3 * X);

mask = Z > 0.5;
nHigh = nnz(mask);
meanHigh = mean(Z(mask))
[rows, cols] = find(mask);
spanR = max(rows) - min(rows)
spanC = max(cols) - min(cols);

counts = histc(Z(:), linspace(-1, 1, 11));
peak = max(counts)

surf(X, Y, Z)
shading interp
colorbar
view(3)
axis tight
xlim([-2 2]); ylim([-2 2]);

smooth = conv2(Z, ones(3) / 9, 'same');
grad = gradient(smooth);
edgeCount = sum(abs(grad(:)) > 0.1)
centre = Z(round(end/2), round(end/2))

grid on
set(gca, 'FontSize', 10)
drawnow
sumAll = sum(Z(:));
avgRow = mean(Z, 2)';
clear X Y
rng default
noise = randn(size(Z));
snr_ = snr(Z, noise)
ok = isequal(size(smooth), size(Z));
