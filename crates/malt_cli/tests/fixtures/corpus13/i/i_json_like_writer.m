function txt = json_like_writer(value, indent)
%JSON_LIKE_WRITER Writes scalars, strings, cells and structs as JSON-like text.
%   Only what the export needs: no escaping beyond quotes.

if nargin < 2
    indent = 0;
end
pad = repmat(' ', 1, 2 * indent);
if ischar(value) || isstring(value)
    txt = ['"' strrep(char(value), '"', '\"') '"'];
elseif isnumeric(value) && isscalar(value)
    txt = num2str(value, 10);
elseif islogical(value)
    if value
        txt = 'true';
    else
        txt = 'false';
    end
elseif iscell(value)
    parts = cell(1, numel(value));
    for k = 1:numel(value)
        parts{k} = json_like_writer(value{k}, indent + 1);
    end
    txt = ['[' strjoin(parts, ', ') ']'];
elseif isstruct(value)
    names = fieldnames(value);
    lines = cell(1, numel(names));
    for k = 1:numel(names)
        lines{k} = sprintf('%s  "%s": %s', pad, names{k}, ...
            json_like_writer(value.(names{k}), indent + 1));
    end
    txt = sprintf('{\n%s\n%s}', strjoin(lines, sprintf(',\n')), pad);
else
    error('json:type', 'unsupported class %s', class(value));
end
end
