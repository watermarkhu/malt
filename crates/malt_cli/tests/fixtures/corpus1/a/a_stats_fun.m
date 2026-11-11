function [m, s, idx] = a_stats_fun(x, options)
%A_STATS_FUN Robust summary statistics for a numeric vector.
%   [M, S, IDX] = A_STATS_FUN(X) returns the mean M and standard
%   deviation S of X after removing outliers, plus the logical index
%   IDX of the kept samples.
%
%   [...] = A_STATS_FUN(X, Name, Value) accepts:
%       Threshold  - outlier threshold in standard deviations (default 3)
%       MaxIter    - maximum number of rejection passes (default 5)

arguments
    x (:,1) double {mustBeReal}
    options.Threshold (1,1) double {mustBePositive} = 3
    options.MaxIter (1,1) double {mustBeInteger, mustBePositive} = 5
end

% Iteratively drop samples further than Threshold sigma from the mean
idx = true(size(x));
n_iter = 0;

while n_iter < options.MaxIter
    n_iter = n_iter + 1;
    mu = mean(x(idx));
    sigma = std(x(idx));
    keep = abs(x - mu) <= options.Threshold * sigma;
    if all(keep(idx) == true)
        break
    end
    idx = idx & keep;
end

m = mean(x(idx));
s = std(x(idx));
m = round_to_precision(m, 6);
s = round_to_precision(s, 6);

if nargout == 0
    print_summary(m, s, nnz(idx), numel(x))
end

end

function y = round_to_precision(x, digits)
% Round X to DIGITS significant digits.
scale = 10^(digits - ceil(log10(abs(x) + eps)));
y = round(x .* scale) ./ scale;
end

function print_summary(m, s, n_kept, n_total)
% Print a short summary to the command window.
fprintf('Mean:  %g\n', m);
fprintf('Std:   %g\n', s);
fprintf('Kept:  %d of %d\n', n_kept, n_total);
end
