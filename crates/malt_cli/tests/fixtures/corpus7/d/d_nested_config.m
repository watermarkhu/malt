function cfg = d_nested_config(base, overrides)
%R3_NESTED_CONFIG Merge nested configuration structs with defaults.
%   CFG = R3_NESTED_CONFIG(BASE, OVERRIDES) returns BASE with every field
%   of OVERRIDES written over it, recursively for struct fields.

if nargin < 2 || isempty(overrides)
    cfg = base;
    return
end

cfg = base;
names = fieldnames(overrides);
for k = 1:numel(names)
    key = names{k};
    newVal = overrides.(key);
    if isstruct(newVal) && isfield(cfg, key) && isstruct(cfg.(key))
        cfg.(key) = d_nested_config(cfg.(key), newVal);
    else
        cfg.(key) = newVal;
    end
end

if isfield(cfg, 'deprecated')
    cfg = rmfield(cfg, 'deprecated');
end

cfg = orderfields(cfg);
leafCount = countLeaves(cfg)
depth = 0;

    function n = countLeaves(s)
        n = 0;
        f = fieldnames(s);
        for i = 1:numel(f)
            v = s.(f{i});
            if isstruct(v)
                n = n + countLeaves(v);
            else
                n = n + 1;
            end
        end
    end
end
