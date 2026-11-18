function [xOpt, fOpt, info] = d_solver_options(x0, method, opts)
%D_SOLVER_OPTIONS Minimise or root-solve with validated options.
%   [xOpt, fOpt] = D_SOLVER_OPTIONS(x0) minimises the Rosenbrock function
%   starting from x0 (2-by-1) with fminsearch.
%
%   [...] = D_SOLVER_OPTIONS(x0, method) with method "fminsearch" or
%   "fzero". For fzero the scalar root of cos(x) - x is searched from x0(1).
%
%   Name-value options, passed as Name=Value:
%     TolX      - tolerance on x (default 1e-8)
%     MaxIter   - iteration limit (default 2000)
%     Display   - "off" or "iter" (default "off")
%     Integrate - also run ode45 on a damped oscillator (default false)

arguments
    x0 (:,1) double {mustBeFinite}
    method (1,1) string {mustBeMember(method, ["fminsearch", "fzero"])} = "fminsearch"
    opts.TolX (1,1) double {mustBePositive} = 1e-8
    opts.MaxIter (1,1) double {mustBeInteger, mustBePositive} = 2000
    opts.Display (1,1) string = "off"
    opts.Integrate (1,1) logical = false
end

info = struct('exitflag', [], 'iterations', NaN);

switch method
    case "fminsearch"
        rosenbrock = @(x) (1 - x(1)).^2 + 100 * (x(2) - x(1).^2).^2;
        searchOpts = optimset('TolX', opts.TolX, 'MaxIter', opts.MaxIter, ...
            'Display', char(opts.Display));
        [xOpt, fOpt, exitflag, output] = fminsearch(rosenbrock, x0, searchOpts);
        info.exitflag = exitflag;
        info.iterations = output.iterations;

    case "fzero"
        g = @(x) cos(x) - x;
        zeroOpts = optimset('TolX', opts.TolX, 'Display', char(opts.Display));
        [xOpt, fOpt, exitflag] = fzero(g, x0(1), zeroOpts);
        info.exitflag = exitflag;
end

% Optional time integration of a damped oscillator for comparison
if opts.Integrate
    zeta = 0.1;
    w0 = 2 * pi;
    rhs = @(t, y) [y(2); -2 * zeta * w0 * y(2) - w0^2 * y(1)];
    odeOpts = odeset('RelTol', 1e-6, 'AbsTol', 1e-9);
    [tSol, ySol] = ode45(rhs, [0 10], [1; 0], odeOpts);
    info.tEnd = tSol(end);
    info.yEnd = ySol(end, :);
end
end
