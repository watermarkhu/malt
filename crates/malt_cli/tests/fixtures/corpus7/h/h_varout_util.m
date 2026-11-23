function varargout = h_varout_util(mode, varargin)
%R3_VAROUT_UTIL Multi-purpose helper: one of several operations, variable outputs.
%   [A, B] = R3_VAROUT_UTIL('minmax', X) returns the min and max of X.
%   [M, S] = R3_VAROUT_UTIL('meanstd', X, DIM) returns mean and std.
%   R3_VAROUT_UTIL('show', X) prints X and returns nothing.

mode = validatestring(mode, {'minmax', 'meanstd', 'show'});
narginchk(2, 3);

x = varargin{1};
switch mode
    case 'minmax'
        narginchk(2, 2)
        varargout{1} = min(x(:));
        varargout{2} = max(x(:));
    case 'meanstd'
        if numel(varargin) < 2
            dim = 1;
        else
            dim = varargin{2};
        end
        varargout{1} = mean(x, dim);
        varargout{2} = std(x, 0, dim);
    case 'show'
        if nargout > 0
            error('r3:varout:noOutput', 'show returns nothing.');
        end
        disp(x)
end

if nargout > 2
    error('r3:varout:tooMany', 'At most two outputs.');
end

info = inputname(2);
unusedA = 1;
unusedB = 'text';
for i = 1:nargout
    v = varargout{i};
    if isempty(v)
        varargout{i} = 0;
    end
end
end
