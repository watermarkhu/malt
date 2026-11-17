%% R1_MONTECARLO  Estimate pi with a parallel loop and an spmd block.
% Needs the Parallel Computing Toolbox for the parfor. Each worker draws
% its own samples; the hits are summed with a reduction variable.

nSamples = 200000;
nWorkers = 4;
rng(42);

hits = 0;
parfor i = 1:nSamples
    x = rand();
    y = rand();
    if x^2 + y^2 <= 1
        hits = hits + 1;
    end
end
piEstimate = 4 * hits / nSamples;

fprintf('pi is about %.4f\n', piEstimate)

spmdHits = zeros(1, nWorkers);
spmd
    localN = nSamples / nWorkers;
    xs = rand(1, localN);
    ys = rand(1, localN);
    spmdHits(labindex) = sum(xs.^2 + ys.^2 <= 1);
end

total = sum(spmdHits{:});
disp(total)

errs = zeros(1, 10);
for k = 1:10
    errs(k) = abs(pi - 4 * sum(rand(1, 1000).^2 + rand(1, 1000).^2 <= 1) / 1000);
end
meanErr = mean(errs);
stdErr = std(errs)

% Second run: a plain loop over batch sizes, no parallel toolbox needed.
batchSizes = [100 1000 10000];
results = cell(size(batchSizes));
for b = 1:numel(batchSizes)
    n = batchSizes(b);
    px = rand(n, 1);
    py = rand(n, 1);
    inside = sum(px.^2 + py.^2 <= 1);
    results{b} = struct('n', n, 'pi', 4 * inside / n);
end

estimates = cellfun(@(r) r.pi, results);
bestIdx = find(abs(estimates - pi) == min(abs(estimates - pi)), 1);
bestN = results{bestIdx}.n;
fprintf('best batch: %d\n', bestN)

history = [];
for s = 1:5
    history = [history, mean(rand(1, 50))];
end
historyMean = mean(history)
