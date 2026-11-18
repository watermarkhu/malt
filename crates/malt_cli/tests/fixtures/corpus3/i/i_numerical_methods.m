function i_numerical_methods()
% I_NUMERICAL_METHODS  root finding, quadrature, ODE and Monte Carlo demos
%   i_numerical_methods()

f = @(x) x.^2 - 2;
fprintf('bisection sqrt2 = %f\n', bisection(f, 0, 2, 1e-8));
fprintf('secant sqrt2 = %f\n', secant(f, 1, 2, 1e-10));

g = @(x) sin(x);
fprintf('simpson = %f\n', simpson(g, 0, pi, 100));
fprintf('fd deriv = %f\n', fd_derivative(g, 0.5, 1e-5));

y = rk4(@(t, y) -y, 1, 0, 1, 0.1);
fprintf('rk4 y(1) = %f (exact %f)\n', y(end), exp(-1));
fprintf('pi approx = %f\n', monte_carlo_pi(10000));

% a small table of squares
for k = 1:3
    fprintf('%d %f\n', k, feval(f, k) + 2);
end
end

function r = bisection(f, a, b, tol)
% halve the bracket until it is smaller than tol
if f(a) * f(b) > 0
    error('no sign change');
end
err = inf;
r = a;
while abs(err) > tol
    r = (a + b) / 2;
    if f(a) * f(r) <= 0
        b = r;
    else
        a = r;
    end
    err = b - a;
end
end

function x = secant(f, x0, x1, tol)
% secant iteration with an iteration cap
err = x1 - x0;
iter = 0;
while abs(err) > tol && iter < 100
    x2 = x1 - feval(f, x1) * (x1 - x0) / (feval(f, x1) - feval(f, x0));
    x0 = x1;
    x1 = x2;
    err = x1 - x0;
    iter = iter + 1;
end
x = x1;
end

function s = simpson(f, a, b, n)
% composite simpson rule
if mod(n, 2) ~= 0
    n = n + 1;
end
h = (b - a) / n;
x = a:h:b;
y = f(x);
s = y(1) + y(end);
for i = 2:n
    if mod(i, 2) == 0
        s = s + 2 * y(i);
    else
        s = s + 4 * y(i);
    end
end
s = s * h / 3;
end

function d = fd_derivative(f, x, h)
% central difference
d = (f(x + h) - f(x - h)) / (2 * h);
end

function ys = rk4(f, y0, t0, t1, h)
% classic fourth order runge kutta
n = round((t1 - t0) / h);
ys = zeros(1, n+1);
ys(1) = y0;
t = t0;
y = y0;
for k = 1:n
    k1 = f(t, y);
    k2 = f(t + h/2, y + h/2 * k1);
    k3 = f(t + h/2, y + h/2 * k2);
    k4 = f(t + h, y + h * k3);
    y = y + h / 6 * (k1 + 2*k2 + 2*k3 + k4);
    t = t + h;
    ys(k+1) = y;
end
end

function p = monte_carlo_pi(n)
% fraction of random points inside the quarter circle
hits = 0;
for i = 1:n
    x = rand; y = rand;
    if x^2 + y^2 <= 1
        hits = hits + 1;
    end
end
p = 4 * hits / n;
end
