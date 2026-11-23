function v = j_persist_cache(key, val)
% PERSIST_CACHE  tiny memo store using persistent state
persistent store hits misses
if isempty(store)
    store = struct();
    hits = 0;
    misses = 0;
end
if nargin == 0
    v = struct('hits', hits, 'misses', misses, 'size', numel(fieldnames(store)));
    return
end
fld = matlab.lang.makeValidName(key);
if nargin == 2
    store.(fld) = val;
    v = val;
    return
end
if isfield(store, fld)
    v = store.(fld);
    hits = hits + 1;
else
    v = [];
    misses = misses + 1;
end
if misses > 100
    store = struct();
    clear j_persist_cache
end
stamp = clock;
fprintf('%d:%02d lookup %s\n', stamp(4), stamp(5), key);
n = length(store);
assignin('base', 'last_cache_key', key);
evalin('base', 'last_cache_key;');
end
