function out = h_symbolic_math(mode, varargin)
%H_SYMBOLIC_MATH Symbolic and numeric helpers for a damped oscillator
%   OUT = H_SYMBOLIC_MATH('symbolic') derives closed forms with the Symbolic Math Toolbox.
%   OUT = H_SYMBOLIC_MATH('numeric', PARAMS) integrates with ode45.
%   OUT = H_SYMBOLIC_MATH('plot') draws the envelope and signal.

if nargin < 1
    mode = 'symbolic';
end

switch lower(mode)
    case 'symbolic'
        out = symbolicPart(varargin{:});
    case 'numeric'
        out = numericPart(varargin{:});
    case 'plot'
        plotDecay();
        out = [];
    otherwise
        error('h_symbolic_math:mode', 'Unknown mode "%s"', mode);
end
end

function out = symbolicPart(varargin)
% TODO: the solve() output ordering changed in R2019, check on older releases
syms t x(t) m c k w0 real
assume(m > 0)
ode = m*diff(x, t, 2) + c*diff(x, t) + k*x == 0;
disp(ode)

% characteristic polynomial
syms s
p = m*s^2 + c*s + k;
roots_s = solve(p == 0, s);
out.roots = roots_s;

% area under the decaying envelope
env = int(exp(-c/(2*m)*t), t, 0, inf);
out.envelopeArea = simplify(env);

% substitute numbers
vals = subs(roots_s, {m, c, k}, {1, 0.5, 4});
out.numericRoots = double(vals);

% handle for fast repeated evaluation
out.fn = matlabFunction(roots_s, 'Vars', {m, c, k});

if nargin > 0 && isnumeric(varargin{1})
    out.check = evalAt(roots_s(1), varargin{1});
end
end

function out = numericPart(params)
if nargin < 1 || ~isstruct(params)
    params = struct('m', 1, 'c', 0.5, 'k', 4, 'x0', 1, 'T', 20);
end
f = @(t, y) [y(2); -(params.c*y(2) + params.k*y(1))/params.m];
opts = odeset('RelTol', 1e-6, 'AbsTol', 1e-9);
[tOut, yOut] = ode45(f, [0 params.T], [params.x0; 0], opts);
out.t = tOut;
out.x = yOut(:, 1);

% first zero crossing by root finding on the interpolant
g = @(t) interp1(tOut, yOut(:,1), t);
if any(yOut(:,1) < 0)
    idx = find(yOut(:,1) < 0, 1);
    out.firstZero = fzero(g, [tOut(idx-1) tOut(idx)]);
else
    out.firstZero = NaN;
end

% numeric path for a symbolic expression if the toolbox is present
if license('test', 'Symbolic_Toolbox')
    % not wired up yet, see TODO in symbolicPart
end
end

function plotDecay()
fplot(@(t) exp(-t), [0 5]);
hold on
fplot(@(t) exp(-t).*cos(2*t), [0 5], '--');
hold off
legend('envelope', 'signal', 'Location', 'northeast');
grid on
end

function y = evalAt(expr, val)
% symbolic or plain function handle, same call site
if isa(expr, 'sym')
    y = double(subs(expr, val));
else
    y = expr(val);
end
end
