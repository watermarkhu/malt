function [total, grid_out] = a_parfor_sim(n_trials, n_steps, seed)
%A_PARFOR_SIM Monte Carlo random walk using parfor.
%   [TOTAL, GRID_OUT] = A_PARFOR_SIM(N_TRIALS, N_STEPS, SEED) runs
%   N_TRIALS random walks of N_STEPS each. TOTAL is the sum of the final
%   positions. GRID_OUT holds the final position of every trial.

if nargin < 3
    seed = 1;
end

rng(seed)
grid_out = zeros(n_trials, 1);
total = 0;

% TOTAL is a reduction variable, GRID_OUT is a sliced output
parfor k = 1:n_trials
    steps = 2 * (rand(n_steps, 1) > 0.5) - 1;
    final_pos = sum(steps);
    grid_out(k) = final_pos;
    total = total + final_pos;
end

fprintf('Mean final position: %.3f\n', total / n_trials)

end

function results = a_parfor_spmd_helper(n_points)
%A_PARFOR_SPMD_HELPER Sum of squares computed on each pool worker.
%   RESULTS = A_PARFOR_SPMD_HELPER(N_POINTS) splits N_POINTS across the
%   workers of the current pool and returns the partial sum per worker.

pool = gcp('nocreate');
if isempty(pool)
    pool = parpool('local', 2);
end
n_workers = pool.NumWorkers;

spmd
    local_n = floor(n_points / n_workers);
    local_vals = linspace(0, 1, local_n)';
    local_sum = sum(local_vals .^ 2);
end

results = zeros(n_workers, 1);
for w = 1:n_workers
    results(w) = local_sum{w};
end

end
