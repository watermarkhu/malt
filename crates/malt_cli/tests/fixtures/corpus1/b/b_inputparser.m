function result = b_inputparser(data, varargin)
%B_INPUTPARSER Smooth a numeric vector with a configurable window.
%   RESULT = B_INPUTPARSER(DATA) applies a moving average with the default
%   window of 5 samples.
%
%   Name-value options:
%       'Window'   positive odd integer (default 5)
%       'Method'   'mean' | 'median' | 'gaussian' (default 'mean')
%       'Padding'  'zero' | 'replicate' (default 'replicate')
%       'Label'    text used in messages (default "series")

narginchk(1, 9);

p = inputParser;
p.FunctionName  = 'b_inputparser';
p.CaseSensitive = false;
p.KeepUnmatched = false;

isWindow = @(x) isnumeric(x) && isscalar(x) && x >= 1 && mod(x, 2) == 1;
isMethod = @(x) any(strcmpi(x, {'mean', 'median', 'gaussian'}));
isText   = @(x) ischar(x) || isstring(x);

addRequired(p, 'data', @(x) isnumeric(x) && isvector(x));
addParameter(p, 'Window', 5, isWindow);
addParameter(p, 'Method', 'mean', isMethod);
addParameter(p, 'Padding', 'replicate', isText);
addParameter(p, 'Label', "series", isText);
parse(p, data, varargin{:});

opts = p.Results;
validateattributes(data, {'numeric'}, {'nonempty', 'real'}, mfilename, 'data');

x = opts.data(:).';
n = numel(x);
h = (opts.Window - 1) / 2;
out = zeros(size(x));

% Pad the signal so that every output sample has a full window
padded = padSignal(x, h, opts.Padding);

try
    switch lower(opts.Method)
        case 'mean'
            for i = 1:n
                out(i) = mean(padded(i:i + 2*h));
            end
        case 'median'
            for i = 1:n
                out(i) = median(padded(i:i + 2*h));
            end
        case 'gaussian'
            kern = exp(-((-h:h).^2) / (2 * (h/2 + eps)^2));
            kern = kern / sum(kern);
            out = conv(padded, kern, 'valid');
    end
catch ME
    fprintf('%s: smoothing failed (%s)\n', char(opts.Label), ME.message);
    rethrow(ME);
end

result = struct('smoothed', out, 'window', opts.Window, ...
    'label', char(opts.Label));

% Report the largest change introduced by the smoothing
delta = max(abs(out - x));
if delta > 10 * std(x)
    warning('b_inputparser:largeChange', ...
        'Maximum change %.3g is large for this series.', delta);
end

end

function xp = padSignal(x, h, mode)
%PADSIGNAL Extend a row vector by h samples on each side.
switch lower(char(mode))
    case 'zero'
        xp = [zeros(1, h), x, zeros(1, h)];
    case 'replicate'
        xp = [repmat(x(1), 1, h), x, repmat(x(end), 1, h)];
    otherwise
        error('b_inputparser:badPadding', ...
            'Padding must be ''zero'' or ''replicate''.');
end
end
