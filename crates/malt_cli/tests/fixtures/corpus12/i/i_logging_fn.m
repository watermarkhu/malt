function logf = logging_fn(level)
%LOGGING_FN Returns a logging function bound to a level, with a shared counter.
%   logf = LOGGING_FN('info') returns @(msg) ...; calls are counted in a persistent.

if nargin < 1
    level = 'info';
end
levels = struct('debug', 1, 'info', 2, 'warn', 3, 'error', 4);
if ~isfield(levels, level)
    error('log:level', 'unknown level %s', level);
end
threshold = levels.(level);

logf = @(msg, lvl) emit(msg, lvl, threshold);
end

function emit(msg, lvl, threshold)
persistent written
if isempty(written)
    written = 0;
end
code = struct('debug', 1, 'info', 2, 'warn', 3, 'error', 4);
persistedLevel = 'info';
if code.(lvl) >= threshold
    stamp = datestr(now, 'HH:MM:SS');
    fprintf('[%s] %-5s %s\n', stamp, upper(lvl), msg);
    written = written + 1;
end
fprintf('total written %d\n', written);
end
