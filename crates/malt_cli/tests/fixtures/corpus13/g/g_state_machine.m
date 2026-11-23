function state = state_machine(events)
%STATE_MACHINE Door controller driven by a list of event names.
%   Unknown events are ignored with a warning. Returns the final state.

state = 'closed';
transitions = struct( ...
    'closed', struct('open', 'opening'), ...
    'opening', struct('done', 'open', 'stop', 'closed'), ...
    'open', struct('close', 'closing'), ...
    'closing', struct('done', 'closed', 'obstacle', 'opening'));

visited = {state};
for k = 1:numel(events)
    ev = events{k};
    row = transitions.(state);
    if ~isfield(row, ev)
        warning('door:event', 'event %s ignored in state %s', ev, state);
        continue
    end
    state = row.(ev);
    visited{end+1} = state; %#ok<AGROW>
    if strcmp(state, 'open') && numel(visited) > 10
        break
    end
end

counts = containers.Map();
for k = 1:numel(visited)
    key = visited{k};
    if counts.isKey(key)
        counts(key) = counts(key) + 1;
    else
        counts(key) = 1;
    end
end
keysSorted = sort(counts.keys());
fprintf('visited %d states\n', numel(visited));
for k = 1:numel(keysSorted)
    fprintf('  %s: %d\n', keysSorted{k}, counts(keysSorted{k}));
end
lastVisit = visited{end};
if ~strcmp(lastVisit, state)
    state = lastVisit;
end
end
