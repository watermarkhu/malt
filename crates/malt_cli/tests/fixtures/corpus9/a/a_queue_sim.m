% Discrete-event simulation of a single-server queue with customer classes.
% Written as a script: edit the parameters below and run.

rng(42)
lambda = 0.8;      % arrival rate per minute
mu = 1.0;          % service rate per minute
horizon = 480      % minutes
classNames = {'walkin', 'booked', 'vip'};
classWeights = [0.6 0.3 0.1];
stats = containers.Map();
for c = 1:numel(classNames)
    stats(classNames{c}) = struct('served', 0, 'waitSum', 0, 'maxWait', 0);
end

t = 0;
queue = [];
busyUntil = 0;
served = 0;
waits = [];
log = {};
nextArrival = exprnd(1/lambda);
while t < horizon
    if nextArrival <= busyUntil || isempty(queue)
        t = nextArrival;
        nextArrival = t + exprnd(1/lambda);
        r = rand;
        cls = find(cumsum(classWeights) >= r, 1);
        queue(end+1) = struct('arrived', t, 'cls', cls);
        if isempty(queue) == 0 && busyUntil <= t
            busyUntil = t;
        end
        continue
    end
    cust = queue(1);
    queue(1) = [];
    startTime = max(busyUntil, cust.arrived);
    wait = startTime - cust.arrived;
    busyUntil = startTime + exprnd(1/mu);
    served = served + 1;
    waits(end+1) = wait;
    key = classNames{cust.cls};
    s = stats(key);
    s.served = s.served + 1;
    s.waitSum = s.waitSum + wait;
    s.maxWait = max(s.maxWait, wait);
    stats(key) = s;
    if mod(served, 50) == 0
        log{end+1} = sprintf('served %d at t=%.1f, queue %d', served, t, numel(queue));
    end
    if wait > 10
        warning('queue:longWait', 'customer waited %.1f min', wait)
    end
end

fprintf('served %d customers\n', served)
fprintf('mean wait %.2f min\n', mean(waits))
fprintf('p95 wait %.2f min\n', prctile(waits, 95))
keysList = keys(stats);
for k = 1:numel(keysList)
    s = stats(keysList{k});
    if s.served > 0
        fprintf('%-8s served=%4d meanWait=%.2f max=%.2f\n', keysList{k}, s.served, s.waitSum/s.served, s.maxWait);
    end
end
cellfun(@disp, log)
longest = max(waits);
idx = find(waits == longest);
unusedCount = numel(idx);
busy = busyUntil - t;
