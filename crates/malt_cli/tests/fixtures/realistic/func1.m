function out = func1(in, varargin)
%FUNC1 Do things.
p = inputParser;
addParameter(p, 'Scale', 1);
parse(p, varargin{:});
scale = p.Results.Scale;
out = [];
for k = 1:numel(in)
    out(end+1) = in(k) * scale; %#ok<AGROW>
end
try
    out = cumsum(out);
catch err
    warning('failed: %s', err.message);
end
switch numel(out)
    case 0
        out = NaN;
    otherwise
        out = out(:);
end
end
