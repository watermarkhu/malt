function out = cell_pipeline(items)
%CELL_PIPELINE Applies a list of named transforms to a cell array of items.
%   out = CELL_PIPELINE(items) returns a cell array with the same size.

steps = {
    'trim',    @(s) strtrim(s)
    'upper',   @(s) upper(s)
    'nodigit', @(s) regexprep(s, '\d', '')
    };

out = items;
for k = 1:size(steps, 1)
    name = steps{k, 1};
    fcn = steps{k, 2};
    out = cellfun(fcn, out, 'UniformOutput', false);
    fprintf('after %s: %d items\n', name, numel(out));
end

lens = cellfun('length', out);
empties = cellfun(@isempty, out);
out(empties) = [];
[~, ord] = sort(lens(~empties));
out = out(ord);
joined = [out{:}];
firstChar = cellfun(@(s) s(1), out(~cellfun(@isempty, out)))
nums = cellfun(@str2double, out, 'ErrorHandler', @(err, s) NaN);
valid = nums(~isnan(nums));
mx = max(valid);
tags = cell(size(out));
for k = 1:numel(out)
    if mod(k, 2) == 0
        tags{k} = 'even';
    else
        tags{k} = 'odd';
    end
end
ok = all(cellfun(@ischar, out));
if ~ok
    error('pipeline:type', 'non-char item');
end
count = numel(tags);
end
