function [x, info] = c_args_block(A, b, opts)
%C_ARGS_BLOCK Solve A*x = b with a damped Jacobi iteration.
%   [X, INFO] = C_ARGS_BLOCK(A, B, OPTS) uses the name-value options in
%   OPTS.Tolerance, OPTS.MaxIter and OPTS.Damping.

arguments
    A (:,:) double {mustBeSquareMatrix}
    b (:,1) double
    opts.Tolerance (1,1) double {mustBePositive} = 1e-6
    opts.MaxIter (1,1) double {mustBeInteger, mustBePositive} = 500
    opts.Damping (1,1) double {mustBeInRange(opts.Damping, 0, 1)} = 0.8
end

n = size(A, 1);
if numel(b) ~= n
    error('c_args_block:size', 'b must have %d elements', n);
end

d = diag(A);
R = A - diag(d);
x = zeros(n, 1);
resid = inf;
iter = 0;

while resid > opts.Tolerance && iter < opts.MaxIter
    xNew = (b - R * x) ./ d;
    x = opts.Damping * xNew + (1 - opts.Damping) * x;
    resid = norm(A * x - b) / norm(b);
    iter = iter + 1;
end

info = struct('iterations', iter, 'residual', resid, ...
    'converged', resid <= opts.Tolerance);
info.label = describeMethod(opts.Damping);

end

function mustBeSquareMatrix(M)
% Argument validator used by the main function
if size(M, 1) ~= size(M, 2)
    error('c_args_block:notSquare', 'Matrix must be square');
end
end

function label = describeMethod(damping)
arguments
    damping (1,1) double
end
if damping == 1
    label = 'Jacobi';
else
    label = 'Damped Jacobi';
end
end
