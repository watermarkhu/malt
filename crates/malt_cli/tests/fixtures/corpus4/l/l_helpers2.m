function tf = l_helpers2(d, radius, varargin)
%L_HELPERS2 Tolerant comparison of distances against a radius.
%   TF = L_HELPERS2(D, RADIUS) returns true where D <= RADIUS + tol.
%   The tolerance keeps points on the boundary inside, which matters
%   when distances come from floating-point arithmetic.
%
%   Options:
%       'Tol'   absolute tolerance, default 1e-12
%       'Label' name used in error messages, default 'distance'

p = inputParser;
addRequired(p, 'd', @isnumeric);
addRequired(p, 'radius', @(r) isnumeric(r) && isscalar(r));
addParameter(p, 'Tol', 1e-12, @(t) isnumeric(t) && t >= 0);
addParameter(p, 'Label', 'distance', @ischar);
parse(p, d, radius, varargin{:});

tol = p.Results.Tol;
tf = d <= radius + tol;
tf = logical(tf);

if any(isnan(tf(:)))
    error('l_helpers2:nan', '%s contains NaN values.', p.Results.Label);
end
end
