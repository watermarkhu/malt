function [result, attempts] = c_retry_util(fn, varargin)
%R1_RETRY_UTIL Call a function handle with retries and backoff.
%   [R, N] = R1_RETRY_UTIL(FN, 'MaxTries', 3, 'Pause', 0.5) calls FN with no
%   arguments until it succeeds or the tries run out. The number of
%   attempts is returned as N.

p = inputParser;
p.FunctionName = 'c_retry_util';
addRequired(p, 'fn', @(x) isa(x, 'function_handle'));
addParameter(p, 'MaxTries', 3, @isnumeric);
addParameter(p, 'Pause', 0.5, @(x) isnumeric(x) && x >= 0);
parse(p, fn, varargin{:});
opts = p.Results;

persistent callCount
if isempty(callCount)
    callCount = 0;
end

result = [];
attempts = 0;
ok = false;
while ~ok && attempts < opts.MaxTries
    attempts = attempts + 1;
    callCount = callCount + 1;
    try
        result = fn();
        ok = true;
    catch ME
        warning('r1:retry:failed', 'Attempt %d failed: %s', attempts, ME.message);
        pause(opts.Pause * attempts);
    end
end

if ~ok
    result = getLastError_(attempts);
end

    function msg = getLastError_(n)
        msg = sprintf('gave up after %d attempts', n);
    end

end
