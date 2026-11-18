function [x, fval, info] = j_optim_old(x0, data, method)
%J_OPTIM_OLD  Fit the decay model with the optimisation toolbox, old style.
%   [X, FVAL] = J_OPTIM_OLD(X0, DATA) fits the two parameter decay model to
%   DATA(:,1) (time) and DATA(:,2) (signal) using fminsearch.
%   METHOD may be 'simplex', 'unc' or 'lsq' (default 'simplex').

if nargin < 3
    method = 'simplex';
end
if nargin < 1 | isempty(x0)
    x0 = [1 0.5];
end

% options as they were set in the 2000 release, names kept as-is
options = optimset('LargeScale', 'off', 'Display', 'iter', ...
                   'MaxFunEvals', 2000, 'MaxIter', 500, ...
                   'TolX', 1e-8, 'TolFun', 1e-8);

% the objective is a string name, the way the old examples did it
objname = 'j_optim_obj';

info = [];
switch method
    case 'simplex'
        [x, fval, exitflag] = fminsearch(objname, x0, options, data);
        info.exitflag = exitflag;

    case 'unc'
        optu = optimset('LargeScale', 'off', 'Display', 'final', ...
                        'GradObj', 'off', 'DerivativeCheck', 'off');
        [x, fval, exitflag, output] = fminunc(@(p) j_optim_obj(p, data), x0, optu);
        info.exitflag = exitflag;
        info.output = output;

    case 'lsq'
        lb = [0 0];
        ub = [10 10];
        optl = optimset('Display', 'iter', 'TolFun', 1e-10);
        [x, resnorm] = lsqnonlin('j_optim_resid', x0, lb, ub, optl, data);
        fval = resnorm;
        info.resnorm = resnorm;

    otherwise
        error(['unknown method ' method])
end

% try a few starting points as well, to check for local minima
starts = [0.5 0.1; 1 1; 2 0.2];
best = inf;
for k = 1:size(starts, 1)
    [xk, fk] = fminsearch(objname, starts(k, :), options, data);
    if fk < best
        best = fk;
        xbest = xk;
    end
end
fprintf('best of the starts: fval %g at %g %g\n', best, xbest(1), xbest(2))

% confidence interval from the jacobian, numerical
J = j_jacobian(@(p) j_optim_resid(p, data), x);
covm = inv(J' * J) * fval / max(1, size(data, 1) - 2);
se = sqrt(diag(covm));
info.se = se;

% plot the fit in the current figure
t = data(:, 1);
model = x(1) * exp(-x(2) * t);
figure(1)
plot(t, data(:, 2), 'o', t, model, '-')
xlabel('time')
ylabel('signal')
legend('data', 'fit')

% keep the parameter history in a global, used by the old report
global OPTIM_HISTORY
OPTIM_HISTORY = [x fval];

% a inline version of the objective for the quick check
obj = inline('p(1)*exp(-p(2)*t)', 'p', 't');
quick = obj(x, t);
disp(['quick check max residual: ' num2str(max(abs(quick - data(:, 2))))])

function f = j_optim_obj(p, data)
% sum of squared residuals for the decay model, scalar output
t = data(:, 1);
y = data(:, 2);
f = sum((y - p(1) * exp(-p(2) * t)).^2);

function r = j_optim_resid(p, data)
% residual vector for lsqnonlin and the jacobian
t = data(:, 1);
y = data(:, 2);
r = y - p(1) * exp(-p(2) * t);

function J = j_jacobian(fun, x)
% forward difference jacobian of a residual function
h = 1e-6;
r0 = feval(fun, x);
J = zeros(length(r0), length(x));
for k = 1:length(x)
    xk = x;
    xk(k) = xk(k) + h;
    J(:, k) = (feval(fun, xk) - r0) / h;
end
