function [t, y, info] = b_ode_solver(fun, tspan, y0, opts)
%B_ODE_SOLVER Fixed-step RK4 integrator with Newton-corrected steps.
%   [T, Y, INFO] = B_ODE_SOLVER(FUN, TSPAN, Y0) integrates the system
%   dy/dt = FUN(t, y) from TSPAN(1) to TSPAN(end), starting at Y0.
%
%   [T, Y, INFO] = B_ODE_SOLVER(FUN, TSPAN, Y0, OPTS) accepts a struct
%   with the optional fields:
%       h        step size                 (default 1e-2)
%       tol      Newton tolerance          (default 1e-10)
%       maxIter  Newton iteration limit    (default 25)
%
%   INFO reports nSteps, maxResidual and totalNewtonIters.

if nargin < 4 || isempty(opts)
    opts = struct();
end
h       = getOpt(opts, 'h', 1e-2);
tol     = getOpt(opts, 'tol', 1e-10);
maxIter = getOpt(opts, 'maxIter', 25);

if numel(tspan) < 2
    error('b_ode_solver:badSpan', 'TSPAN needs at least two entries.');
end

tEnd   = tspan(end);
nSteps = ceil((tEnd - tspan(1)) / h);
nState = numel(y0);

% Preallocate the output trajectory
t = zeros(nSteps + 1, 1);
y = zeros(nSteps + 1, nState);
t(1)    = tspan(1);
y(1, :) = y0(:).';

info = struct('nSteps', nSteps, 'maxResidual', 0, 'totalNewtonIters', 0);

k = 1;
while t(k) < tEnd - eps(tEnd)
    hk = min(h, tEnd - t(k));
    yk = y(k, :).';

    % Explicit RK4 predictor
    k1 = fun(t(k), yk);
    k2 = fun(t(k) + hk/2, yk + hk/2 * k1);
    k3 = fun(t(k) + hk/2, yk + hk/2 * k2);
    k4 = fun(t(k) + hk,   yk + hk * k3);
    yPred = yk + hk/6 * (k1 + 2*k2 + 2*k3 + k4);

    % Newton correction on the implicit trapezoidal residual
    f0 = fun(t(k), yk);
    residual = @(z) z - yk - hk/2 * (f0 + fun(t(k) + hk, z));
    [yNew, nIt, res] = newtonSolve(residual, yPred, tol, maxIter);

    info.totalNewtonIters = info.totalNewtonIters + nIt;
    info.maxResidual = max(info.maxResidual, res);

    k = k + 1;
    t(k)    = t(k - 1) + hk;
    y(k, :) = yNew.';

    % Stop early if the solution has blown up
    if any(~isfinite(yNew)) || any(abs(yNew) > 1e8)
        warning('b_ode_solver:blowup', 'Solution diverged at t = %g.', t(k));
        break
    end
end

% Drop the unused preallocated rows
t = t(1:k);
y = y(1:k, :);
info.lastStep = t(end) - t(end-1);
info.nSteps   = k - 1;

end

function [z, nIt, res] = newtonSolve(F, z, tol, maxIter)
%NEWTONSOLVE Newton iteration with a finite-difference Jacobian.
nIt = 0;
res = inf;
while nIt < maxIter
    r = F(z);
    res = norm(r, inf);
    if res < tol
        break
    end
    J = numericJacobian(F, z, r);
    z = z - J \ r;
    nIt = nIt + 1;
end
end

function J = numericJacobian(F, z, r0)
%NUMERICJACOBIAN Forward-difference Jacobian of F at z.
n = numel(z);
J = zeros(n);
eps0 = sqrt(eps);
for j = 1:n
    dzj = eps0 * max(1, abs(z(j)));
    zp = z;
    zp(j) = zp(j) + dzj;
    J(:, j) = (F(zp) - r0) / dzj;
end
end

function v = getOpt(s, name, default)
%GETOPT Read a field from a struct with a fallback default.
if isfield(s, name) && ~isempty(s.(name))
    v = s.(name);
else
    v = default;
end
end
