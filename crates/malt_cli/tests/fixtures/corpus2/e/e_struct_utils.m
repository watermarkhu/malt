function out = e_struct_utils(varargin)
%E_STRUCT_UTILS Helpers for nested configuration structs.
%   out = e_struct_utils(base, override, ...) deep-merges structs, with
%   later arguments taking priority. Nested structs are merged field by
%   field; everything else is replaced.
%
%   e_struct_utils('summary', s) prints a one-line summary per field.
%   [names, values] = e_struct_utils('flatten', s) lists leaf fields.

if nargin == 0
    error('e_struct_utils:noInput', 'Give at least one struct.');
end

first = varargin{1};
if ischar(first) || isstring(first)
    switch lower(char(first))
        case 'summary'
            printSummary(varargin{2});
            out = [];
        case 'flatten'
            [names, values] = flattenStruct(varargin{2}, '');
            out = struct('names', {names}, 'values', {values});
        otherwise
            error('e_struct_utils:badCommand', 'Unknown command %s', first);
    end
    return
end

% Merge all inputs from left to right
out = first;
for k = 2:numel(varargin)
    out = deepMerge(out, varargin{k});
end
end

function merged = deepMerge(base, extra)
% Recursively merge extra into base.
merged = base;
fields = fieldnames(extra);
for k = 1:numel(fields)
    name = fields{k};
    if isfield(merged, name) && isstruct(merged.(name)) && isstruct(extra.(name))
        merged.(name) = deepMerge(merged.(name), extra.(name));
    else
        merged.(name) = extra.(name);
    end
end
end

function printSummary(s)
names = fieldnames(s);
for k = 1:numel(names)
    val = s.(names{k});
    if isnumeric(val)
        desc = sprintf('%s', mat2str(val));
    elseif ischar(val)
        desc = val;
    elseif iscell(val)
        desc = sprintf('cell %dx%d', size(val, 1), size(val, 2));
    elseif isstruct(val)
        desc = sprintf('struct with %d fields', numel(fieldnames(val)));
    else
        desc = class(val);
    end
    fprintf('%-12s %s\n', names{k}, desc);
end
end

function [names, values] = flattenStruct(s, prefix)
names = {};
values = {};
fields = fieldnames(s);
for k = 1:numel(fields)
    key = fields{k};
    if ~isempty(prefix)
        fullKey = [prefix '.' key];
    else
        fullKey = key;
    end
    val = s.(key);
    if isstruct(val)
        [subNames, subValues] = flattenStruct(val, fullKey);
        names = [names, subNames]; %#ok<AGROW>
        values = [values, subValues]; %#ok<AGROW>
    else
        names{end+1} = fullKey; %#ok<AGROW>
        values{end+1} = val; %#ok<AGROW>
    end
end
end
