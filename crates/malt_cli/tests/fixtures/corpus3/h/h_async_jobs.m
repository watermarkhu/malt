function results = h_async_jobs(items, opts)
%H_ASYNC_JOBS Run per-item jobs on the pool, with a sequential fallback
%   RESULTS = H_ASYNC_JOBS(ITEMS) processes each cell element of ITEMS.
%   RESULTS = H_ASYNC_JOBS(ITEMS, OPTS) with OPTS fields:
%     UseParallel  logical, default true
%     Timeout      seconds to wait for futures, default 300
%     Verbose      logical, default true

if nargin < 2 || isempty(opts)
    opts = struct('UseParallel', true, 'Timeout', 300, 'Verbose', true);
end
if ~isfield(opts, 'UseParallel'), opts.UseParallel = true; end
if ~isfield(opts, 'Timeout'), opts.Timeout = 300; end
if ~isfield(opts, 'Verbose'), opts.Verbose = true; end

n = numel(items);
results = cell(n, 1);
pool = gcp('nocreate');

if opts.UseParallel && isempty(pool)
    try
        pool = parpool('local', 4);
    catch ME
        warning('h_async_jobs:noPool', 'Could not start pool (%s). Falling back.', ME.message);
        opts.UseParallel = false;
    end
end

if ~opts.UseParallel
    % sequential fallback, kept for machines without the Parallel Computing Toolbox
    for i = 1:n
        results{i} = processItem(items{i});
        if opts.Verbose
            fprintf('seq %d/%d\n', i, n);
        end
    end
    return
end

% progress counter via DataQueue, the callback runs on the client
tracker = containers.Map('KeyType', 'char', 'ValueType', 'double');
tracker('done') = 0;
q = parallel.pool.DataQueue;
afterEach(q, @(~) bump(tracker, n, opts.Verbose));

% parfor over all items
parfor i = 1:n
    results{i} = processItem(items{i}); %#ok<PFBNS>
    send(q, i);
end

% asynchronous futures for the first few items, cancel the rest on timeout
futs = parallel.FevalFuture.empty;
for i = 1:min(n, 8)
    futs(end+1) = parfeval(pool, @processItem, 1, items{i}); %#ok<AGROW>
end
for i = 1:numel(futs)
    try
        out = fetchOutputs(futs(i));
        results{i} = out{1};
    catch err
        fprintf(2, 'future %d failed: %s\n', i, err.message);
    end
end
pending = futs(strcmp({futs.State}, 'running') | strcmp({futs.State}, 'queued'));
if ~isempty(pending)
    cancel(pending);
end

% one batch job for a long item, writes its result back when finished
if opts.Verbose && n > 100
    job = batch(@processItem, 1, {items{1}}, 'Pool', 4, 'AutoAddClientPath', true);
    wait(job, 'finished', opts.Timeout);
    if ~isempty(job.Error)
        disp(job.Error.message);
    end
    delete(job);
end

% spmd: split work across workers, only if the pool is really there
if ~isempty(gcp('nocreate'))
    spmd (2)
        partial = sum(cellfun(@numel, items(labindex:2:end)));
    end
    total = partial{1} + partial{2};
    fprintf('spmd total %d\n', total);
end
end

function r = processItem(item)
% per-item work, deliberately trivial
if ischar(item) || isstring(item)
    r = struct('name', char(item), 'len', strlength(item));
elseif isnumeric(item)
    r = struct('name', 'num', 'len', numel(item));
else
    r = [];
end
pause(0.01);
end

function bump(tracker, total, verbose)
tracker('done') = tracker('done') + 1;
if verbose
    fprintf('progress %3.0f%%\n', 100 * tracker('done') / total);
end
end
