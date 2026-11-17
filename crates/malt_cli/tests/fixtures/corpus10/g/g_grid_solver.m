%% Steady-state heat on a plate: Jacobi iteration and a sanity check with conv2
nx = 41; ny = 31;
[X, Y] = meshgrid(linspace(0, 1, nx), linspace(0, 1, ny));
T = zeros(ny, nx);
T(1, :) = 100;            % hot top edge
T(end, :) = 0;
mask = false(size(T));
mask(10:20, 15:25) = true;   % insulated block
Tnew = T;
tol = 1e-4;
maxIter = 5000;
resid = inf;
iter = 0;
while resid > tol && iter < maxIter
    iter = iter + 1;
    Tnew(2:end-1, 2:end-1) = 0.25 * (T(1:end-2, 2:end-1) + T(3:end, 2:end-1) + ...
        T(2:end-1, 1:end-2) + T(2:end-1, 3:end));
    Tnew(mask) = 50;
    resid = max(abs(Tnew(:) - T(:)));
    T = Tnew;
end
fprintf('converged after %d iterations, residual %.2e\n', iter, resid);

kernel = [0 1 0; 1 -4 1; 0 1 0];
lap = conv2(T, kernel, 'same');
interior = lap(2:end-1, 2:end-1);
maxLap = max(abs(interior(:)))

[~, hotIdx] = max(T(:));
[hotRow, hotCol] = ind2sub(size(T), hotIdx);
flux = trapz(X(1, :), T(2, :))
p = polyfit(X(1, :), T(15, :), 2);
roots_p = roots(p);
dT = gradient(T);
coolSpots = find(T < 10 & ~mask);
numCool = numel(coolSpots);
Tvec = T(:)';
sorted = sort(Tvec, 'descend');
top5 = sorted(1:5);
pos = Tvec(Tvec > 50);
meanPos = mean(pos)
nanCount = sum(isnan(T(:)))
