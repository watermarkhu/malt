function [out, npk] = g_old_toolbox(x, varargin)
%G_OLD_TOOLBOX  smoothing and peak utilities (v0.4)
%
%   OUT = G_OLD_TOOLBOX(X)
%   OUT = G_OLD_TOOLBOX(X, 'mode', 'fast'|'full', 'thresh', T)
%   [OUT, NPK] = G_OLD_TOOLBOX(...)
%
%   Written for the 2008 EEG project. Works on row or column vectors.
%   Returns a struct with fields .smooth, .peaks, .idx
%
%   Known problems: slow on long vectors, peak finding is naive,
%   no check for NaN. Use at own risk.
%
%   See also: smooth, findpeaks (not in 2008 toolbox)

if nargin < 1
    error('g_old_toolbox:noinput', 'Not enough input arguments.');
end
mode = 'full';
thresh = 0.5;
k = 1;
while k <= length(varargin)
    opt = varargin{k};
    if strcmp(opt, 'mode')
        mode = varargin{k+1};
        k = k + 2;
    elseif strcmp(opt, 'thresh')
        thresh = varargin{k+1};
        k = k + 2;
    else
        k = k + 1;
    end
end
if size(x,1) == 1
    x = x';
end
n = size(x,1);
if isempty(find(x > thresh, 1))
    disp('no values above threshold')
end

sm = zeros(n,1);
if strcmp(mode, 'fast')
    w = 3;
else
    w = 7;
end
for i = 1:n
    lo = max(1, i-w);
    hi = min(n, i+w);
    sm(i) = mean(x(lo:hi));
end

pk = [];
idx = [];
for i = 2:n-1
    if x(i) > x(i-1) & x(i) > x(i+1) & x(i) > thresh
        pk(end+1) = x(i);
        idx(end+1) = i;
    end
end
if any(any(isnan(x)))
    warning('NaN in input')
end

% residual, allocated after use (fine in 2008)
res = zeros(n,1);
res = x - sm;
scale = local_scale(x);
res = res / scale;

out.smooth = sm;
out.peaks = pk;
out.idx = idx;
out.resid = res;
npk = length(pk);
if nargout > 1
    disp(['peaks found: ' num2str(npk)])
end

% old code for the 2-column case, not used any more
% if size(x,2) == 2
%     y = x(:,1) + x(:,2);
% end
end

function s = local_scale(x)
% scale factor so that the largest value is one
s = max(abs(x(:)));
if s == 0
    s = 1;
end
end
