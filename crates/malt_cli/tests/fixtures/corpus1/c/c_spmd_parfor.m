%% Parallel idioms for the Monte Carlo study
% Uses parfor for the sweep, spmd for a distributed check and parfeval for
% the background job. Runs on a pool if one is open, serially otherwise.

clear; clc

nSamples = 2000;
x = rand(nSamples, 1);

%% parfor with a reduction and a temporary variable
total = 0;
results = zeros(1, nSamples);
parfor i = 1:nSamples
    temp = sin(x(i)) * cos(x(i));
    results(i) = temp;
    total = total + x(i);
end
fprintf('Reduction total: %.4f\n', total);

%% parfor calling a helper function
scores = zeros(1, nSamples);
parfor i = 1:nSamples
    scores(i) = scoreSample(x(i), 0.3);
end
meanScore = mean(scores);

%% spmd block: each worker handles its own chunk
if isempty(gcp('nocreate'))
    fprintf('No pool open, running spmd on the client\n');
end
spmd
    chunk = labindex:numlabs:nSamples;
    localSum = sum(x(chunk));
    fprintf('Worker %d of %d handled %d samples\n', labindex, numlabs, numel(chunk));
end
globalSum = sum([localSum{:}]);

%% Background job with parfeval
f = parfeval(@sumOfSquares, 1, x);
% ... other work here ...
resultSquares = fetchOutputs(f);
fprintf('Sum of squares: %.4f\n', resultSquares);

%% Cancel the job if it is still running
if ~strcmp(f.State, 'finished')
    cancel(f);
end

% Summary table of the parallel run
summary = struct('samples', nSamples, 'mean', meanScore, ...
    'total', total, 'spmdSum', globalSum);
disp(summary)

%% Local helpers
function s = scoreSample(v, alpha)
    s = alpha * v + (1 - alpha) * v^2;
end

function q = sumOfSquares(v)
    q = sum(v.^2);
end
