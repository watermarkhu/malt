function out = h_error_heavy(items, strict)
%R4_ERROR_HEAVY Validate a list of items and report problems with messages.
%   Mixes try/catch variants, rethrow, throw and error with sprintf messages.

if nargin < 2
    strict = false;
end

out = struct('valid', 0, 'problems', {{}});
status = [];
for k = 1:numel(items)
    item = items{k};
    status = 0;
    try
        if ~isnumeric(item)
            error('r4:items:type', 'item %d is a %s', k, class(item));
        end
        if any(isnan(item(:)))
            error('r4:items:nan', 'item %d has NaN values', k);
        end
        out.valid = out.valid + 1;
    catch ME1
        status = 1;
        out.problems{end+1} = ME1.message;
        if strict
            rethrow(ME1);
        end
    end
    if status == 1
        continue
    end
end

err = [];
try
    error('r4:items:summary', sprintf('%d problems', numel(out.problems)));
catch err
end
if ~isempty(err)
    disp(err.message)
end

last = 0;
try
    error('r4:x:y', 'plain text message %s', 'here');
catch last
end

try
    error(sprintf('bare %d', 3));
catch ME2
    disp(ME2.message)
end

try
    z = 1 / 0;
    throw(MException('r4:items:inf', 'infinite %g', z));
catch ME3
    out.problems{end+1} = ME3.identifier;
end

result = 0;
try
    result = numel(items);
catch
    result = -1;
end
out.count = result;
end
