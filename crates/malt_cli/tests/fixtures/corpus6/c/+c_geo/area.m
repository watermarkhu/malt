function a = area(shape, varargin)
% AREA  Area of a simple shape, dispatched on the shape name.
%   A = R2GEO.AREA('circle', R)
%   A = R2GEO.AREA('rect', W, H)
%   A = R2GEO.AREA('tri', B, H)
%   A = R2GEO.AREA('ellipse', RA, RB)
%   A = R2GEO.AREA('polygon', XY)
%
%   Inputs are not checked for size beyond the number of arguments.

if ~ischar(shape) && ~isstring(shape)
    error('r2geo:area:badShape', 'Shape must be text.');
end

switch lower(char(shape))
    case 'circle'
        r = varargin{1};
        a = pi * r^2;
    case {'rect', 'rectangle'}
        w = varargin{1};
        h = varargin{2};
        a = w * h;
    case 'tri'
        a = 0.5 * varargin{1} * varargin{2};
    case 'ellipse'
        ra = varargin{1};
        rb = varargin{2};
        a = pi * ra * rb;
    case 'polygon'
        xy = varargin{1};
        a = polyarea(xy(:, 1), xy(:, 2));
    otherwise
        error('r2geo:area:unknownShape', 'Unknown shape "%s".', shape)
end

if a < 0
    a = abs(a)
end

extra = nargin - 1
end
