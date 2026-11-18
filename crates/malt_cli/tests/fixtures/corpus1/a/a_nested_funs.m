function [result, log_fn] = a_nested_funs(data, factor)
%A_NESTED_FUNS Apply a transformation with nested helpers.
%   Nested functions read and write the variables of the parent
%   function, so TOTAL and CALLS are shared with the helpers.
%   RESULT is the transformed data. LOG_FN is a handle that prints
%   the running totals.

if nargin < 2
    factor = 2;
end

total = 0;
calls = 0;

% Anonymous functions and cellfun/arrayfun with non-uniform output
scale_fn = @(v) v * factor;
names = {'alpha', 'beta', 'gamma'};
upper_names = cellfun(@(s) upper(s), names, 'UniformOutput', false);
name_lengths = cellfun(@numel, names);
squares = arrayfun(@(v) v.^2, data, 'UniformOutput', false);

result = apply_all(data);
log_fn = @report;

fprintf('Processed %d names, longest is %d characters\n', ...
        numel(upper_names), max(name_lengths))

    function out = apply_all(x)
        % Apply STEP to every element of X
        out = zeros(size(x));
        for k = 1:numel(x)
            out(k) = step(x(k));
        end
    end

    function y = step(v)
        calls = calls + 1;
        total = total + v;
        y = v * factor;
    end

    function report()
        fprintf('Calls: %d, running total: %g\n', calls, total);
    end
end
