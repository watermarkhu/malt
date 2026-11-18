function h = timer_panel(period)
%TIMER_PANEL Starts a timer that refreshes a status line every period seconds.
%   h = TIMER_PANEL(period) returns the timer object. Call stop(h) and delete(h) to end it.

if nargin < 1
    period = 2;
end
state = struct('ticks', 0, 'started', datetime('now'), 'history', []);

h = timer('ExecutionMode', 'fixedRate', 'Period', period, ...
    'TimerFcn', @(src, evt) onTick(src, evt, state), ...
    'ErrorFcn', @onError, 'StopFcn', @(~, ~) disp('panel stopped'));
start(h)
end

function onTick(src, ~, state)
    state.ticks = state.ticks + 1;
    elapsed = datetime('now') - state.started;
    if elapsed > duration(0, 30, 0)
        stop(src)
    end
    msg = sprintf('tick %d after %s', state.ticks, char(elapsed));
    disp(msg)
end

function onError(src, evt)
    warning('panel:tick', 'timer error: %s', evt.Data.message)
    stop(src)
end

function out = format_schedule(times)
% Formats a list of datetime values as HH:MM strings.
out = cell(size(times));
for i = 1:numel(times)
    out{i} = datestr(times(i), 'HH:MM');
end
end

function days = days_between(d1, d2)
days = days(d2 - d1);
end
