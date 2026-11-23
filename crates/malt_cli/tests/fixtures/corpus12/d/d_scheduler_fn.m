function queue = scheduler_fn(jobs, policy)
%SCHEDULER_FN Orders jobs by a policy: 'fifo', 'sjf' or 'priority'.
%   jobs is a struct array with fields name, arrival, length and priority.

if nargin < 2 || isempty(policy)
    policy = 'fifo';
end

switch lower(policy)
    case 'fifo'
        [~, order] = sort([jobs.arrival]);
    case 'sjf'
        [~, order] = sort([jobs.length]);
    case 'priority'
        [~, order] = sort([jobs.priority], 'descend');
    otherwise
        error('scheduler:policy', 'unknown policy: %s', policy);
end
pending = jobs(order);

clock = 0;
queue = struct('name', {}, 'start', {}, 'finish', {});
while ~isempty(pending)
    job = pending(1);
    pending(1) = [];
    start = max(clock, job.arrival);
    finish = start + job.length;
    queue(end+1) = struct('name', job.name, 'start', start, 'finish', finish);
    clock = finish;
    if clock > 100
        warning('scheduler:late', 'queue overran at %g', clock);
        break
    end
end

waits = [queue.start] - [jobs(order).arrival];
avgWait = mean(waits(1:numel(queue)));
fprintf('%d jobs, average wait %.2f\n', numel(queue), avgWait);
makespan = queue(end).finish;
extra = numel(jobs) - numel(queue);
if extra > 0
    queue(end+1:end+extra) = struct('name', {}, 'start', {}, 'finish', {});
end
end
