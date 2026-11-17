%% Parameter sweep of a damped oscillator, run with a parfor loop
% Needs the Parallel Computing Toolbox; falls back to a serial for loop when no pool.

omegas = linspace(0.5, 3.0, 26);
zeta = 0.1;
amp = zeros(size(omegas));
peakTime = zeros(size(omegas));
options = struct('tEnd', 40, 'dt', 0.01);

if isempty(gcp('nocreate'))
    useParallel = false;
else
    useParallel = true;
end

parfor k = 1:numel(omegas)
    w = omegas(k);
    t = 0:options.dt:options.tEnd;
    x = exp(-zeta*w*t) .* cos(w*sqrt(1-zeta^2)*t);
    [amp(k), idx] = max(abs(x(t > 10)));
    tt = t(t > 10);
    peakTime(k) = tt(idx);
end

[~, bestIdx] = max(amp);
fprintf('largest tail amplitude %.4f at omega %.3f\n', amp(bestIdx), omegas(bestIdx));

results = cell(1, 4);
spmd
    part = labindex;
    results{part} = part^2;
end

parfor j = 1:3
    tmp = j * 2;
end

sweepTable = [omegas(:), amp(:), peakTime(:)];
mask = sweepTable(:, 2) > 0.05;
selected = sweepTable(mask, :);
if ~isempty(selected)
    disp(selected(1:min(5, size(selected, 1)), :))
end

total = 0;
for q = 1:numel(omegas)
    total = total + amp(q);
end
avgAmp = total / numel(omegas);
meanAmp = mean(amp);
