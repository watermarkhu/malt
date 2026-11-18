function [len, bbox, simp] = polyline_tools(pts, opts)
%POLYLINE_TOOLS Length, bounding box and simplification of a 2-D polyline.
%   [len, bbox, simp] = POLYLINE_TOOLS(pts) takes an N-by-2 array of points.
%   opts.tol sets the Douglas-Peucker tolerance (default 0.5).

if nargin < 2
    opts = struct();
end
if ~isfield(opts, 'tol')
    opts.tol = 0.5;
end
validateattributes(pts, {'numeric'}, {'2d', 'ncols', 2, 'nonempty'}, mfilename, 'pts')

len = segLength(pts);
bbox = [min(pts, [], 1), max(pts, [], 1)];
simp = dpSimplify(pts, opts.tol);
fprintf('%d points -> %d after simplification\n', size(pts, 1), size(simp, 1));

    function L = segLength(p)
        d = diff(p, 1, 1);
        L = sum(sqrt(sum(d.^2, 2)));
    end

    function out = dpSimplify(p, tol)
        n = size(p, 1);
        if n <= 2
            out = p;
            return
        end
        a = p(1, :);
        b = p(end, :);
        ab = b - a;
        nrm = norm(ab);
        dist = zeros(n, 1);
        for i = 2:n-1
            ap = p(i, :) - a;
            if nrm == 0
                dist(i) = norm(ap);
            else
                dist(i) = abs(ab(1)*ap(2) - ab(2)*ap(1)) / nrm;
            end
        end
        [dmax, imax] = max(dist);
        if dmax > tol
            left = dpSimplify(p(1:imax, :), tol);
            right = dpSimplify(p(imax:end, :), tol);
            out = [left(1:end-1, :); right];
        else
            out = [a; b];
        end
    end

end

function scaled = rescale_pts(pts, s)
% Helper kept outside the main function: scales about the origin.
scaled = pts * s;
end
