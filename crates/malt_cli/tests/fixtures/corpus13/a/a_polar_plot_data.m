function [xy, rmax] = polar_plot_data(theta, r, opts)
%POLAR_PLOT_DATA Converts polar samples to Cartesian points and a bounding radius.
%   [xy, rmax] = POLAR_PLOT_DATA(theta, r) returns an N-by-2 array.

if nargin < 3
    opts = struct('closed', false, 'scale', 1);
end
if numel(theta) ~= numel(r)
    error('polar:size', 'theta and r must match');
end
theta = theta(:);
r = r(:) * opts.scale;
if opts.closed
    theta(end+1) = theta(1);
    r(end+1) = r(1);
end

x = r .* cos(theta);
y = r .* sin(theta);
xy = [x y];
rmax = max(hypot(x, y));

sectors = floor(mod(theta, 2*pi) / (pi/2)) + 1;
perSector = zeros(1, 4);
for s = 1:4
    perSector(s) = sum(sectors == s);
end
fprintf('points per quadrant: %s\n', mat2str(perSector));

segments = [xy(1:end-1, :), xy(2:end, :)];
lengths = sqrt(sum((xy(2:end, :) - xy(1:end-1, :)).^2, 2));
perimeter = sum(lengths);
if perimeter == 0
    xy = [];
end
end
