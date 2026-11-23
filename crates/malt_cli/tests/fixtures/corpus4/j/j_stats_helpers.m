function out = j_stats_helpers(data, method, nbins)
%J_STATS_HELPERS  Descriptive statistics helpers used in the lab reports.
%   OUT = J_STATS_HELPERS(DATA) returns mean, std and a correlation matrix.
%   OUT = J_STATS_HELPERS(DATA, METHOD) selects 'mean', 'median', 'rank'
%   or 'hist'.  OUT = J_STATS_HELPERS(DATA, METHOD, NBINS) sets the bins.
%
%   Mostly written for the Statistics Toolbox of the 2000 era.

if nargin < 2
    method = 'mean';
end
if nargin < 3
    nbins = 10;
end

out = [];
if isempty(data)
    disp('no data given')
    return
end

if isstr(method)
    method = lower(method);
end

% the group of methods the lab accepts, as a cell array of strings
allowed = {'mean', 'median', 'rank', 'hist', 'corr'};
if isempty(find(strcmp(method, allowed)))
    error(['unknown method ' method])
end

switch method
    case 'mean'
        out.mean = mean(data);
        out.std = std(data, 1);
        out.n = length(data);
    case 'median'
        out.median = median(data);
        out.mode = mode(round(data));
    case 'rank'
        out = j_rank_data(data);
    case 'hist'
        [n, xout] = hist(data, nbins);
        out.counts = n;
        out.centres = xout;
        bar(xout, n)
    case 'corr'
        out = j_corr_table(data);
end

% the old code kept the summary in a global for the report script
global STATS_SUMMARY
STATS_SUMMARY = out;

% summary line printed to the command window
if isstruct(out) & isfield(out, 'mean')
    fprintf('mean = %8.4f  std = %8.4f  n = %d\n', out.mean, out.std, out.n)
end

% a quick check of the sample size with the old idiom
if exist('out') & ~isempty(out)
    disp('summary available')
end

% outlier flags, three sigma rule, written with plain loops
if isnumeric(data) & ~isempty(data)
    m = mean(data(:));
    s = std(data(:));
    flags = zeros(size(data));
    for i = 1:size(data, 1)
        for j = 1:size(data, 2)
            if abs(data(i, j) - m) > 3 * s
                flags(i, j) = 1;
            end
        end
    end
    nout = sum(flags(:));
    if nout > 0
        fprintf('%d possible outliers\n', nout)
    end
end

% pairs of columns, correlation with corrcoef
if size(data, 2) == 2
    cc = corrcoef(data(:, 1), data(:, 2));
    out.r = cc(1, 2);
end
end

function r = j_rank_data(x)
% average ranks for ties, the way the old stats book describes it
x = x(:);
n = length(x);
[s, idx] = sort(x);
r = zeros(n, 1);
i = 1;
while i <= n
    j = i;
    while j < n & s(j + 1) == s(i)
        j = j + 1;
    end
    avg = (i + j) / 2;
    for k = i:j
        r(idx(k)) = avg;
    end
    i = j + 1;
end
end

function C = j_corr_table(M)
% correlation table for all pairs of columns, with the loop kept simple
p = size(M, 2);
C = zeros(p);
for i = 1:p
    for j = 1:p
        cc = corrcoef(M(:, i), M(:, j));
        C(i, j) = cc(1, 2);
    end
end
if any(isnan(C(:)))
    disp('some columns are constant, correlation undefined')
end
end

function y = j_trim_mean(x, pct)
% trimmed mean, with the percentage given as a fraction
if nargin < 2, pct = 0.1; end
x = sort(x(:));
k = floor(pct * length(x));
y = mean(x(k+1:end-k));
end

function z = j_zscore_old(x)
% zscore as the old toolbox computed it, population std
z = (x - nanmean(x)) ./ std(x, 1);
end
