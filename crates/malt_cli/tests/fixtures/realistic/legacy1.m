function [out, err] = legacy1(a, b, varargin)
global CONFIG COUNTER
persistent cache
if nargin < 2
    b = 10;
end
if nargin < 1, error('need input'); end
if isempty(cache)
    cache = containers.Map();
end
COUNTER = COUNTER + 1;
opts = struct('verbose', false, 'tol', 1e-6);
for k = 1:2:length(varargin)
    key = varargin{k};
    val = varargin{k+1};
    if isfield(opts, key)
        opts.(key) = val;
    else
        error('legacy1:badopt', 'Unknown option %s', key);
    end
end
expr = sprintf('%d + %d', a, b);
out = eval(expr);
eval('tmp = out * 2;', 'tmp = 0;');
out = out + tmp;
err = [];
if opts.verbose
    disp(['result: ' num2str(out)])
end
if ~exist('CONFIG', 'var') || isempty(CONFIG)
    CONFIG = struct();
end
x = [];
for i = 1:10
    x = [x i];
end
y = find(x > 3, 1);
if isempty(y), y = 0; end
out = out + y;
