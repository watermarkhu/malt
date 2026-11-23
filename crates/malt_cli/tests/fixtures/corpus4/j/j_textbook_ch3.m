% j_textbook_ch3.m
% Companion code for Chapter 3 (root finding and simple ODE integration)
% From the numerical methods course notes, 1998 edition, lightly updated 2003.
% Run this script from the directory that holds the course data files.

clc
clear all
close all

format compact
rand('state', 0);            % reproducible numbers for the exercises

% ---- Section 3.1: bisection ------------------------------------------
f = inline('x.^3 - 2*x - 5', 'x');
a = 2;
b = 3;
tol = 1e-6;
maxit = 100;
it = 0;
while (b - a) / 2 > tol & it < maxit
    c = (a + b) / 2;
    if sign(f(c)) == sign(f(a))
        a = c;
    else
        b = c;
    end
    it = it + 1;
end
root_bisect = (a + b) / 2;
disp(sprintf('bisection root = %12.8f after %d steps', root_bisect, it))

% ---- Section 3.2: Newton iteration -----------------------------------
df = inline('3*x.^2 - 2', 'x');
x = 2.5;
for k = 1:20
    dx = f(x) / df(x);
    x = x - dx;
    if abs(dx) < tol
        break
    end
end
disp(sprintf('newton root   = %12.8f after %d steps', x, k))
fprintf('residual          = %g\n', f(x))

% ---- Section 3.3: secant method, trace kept in a global --------------
global SECANT_TRACE
SECANT_TRACE = [];
x0 = 2;
x1 = 3;
for k = 1:30
    x2 = x1 - f(x1) * (x1 - x0) / (f(x1) - f(x0));
    SECANT_TRACE(end+1) = x2;
    x0 = x1;
    x1 = x2;
    if abs(x1 - x0) < tol, break, end
end
disp(['secant root   = ' num2str(x1)])
disp(['trace length  = ' int2str(length(SECANT_TRACE))])

% ---- Section 3.4: explicit Euler for y' = -2*y + sin(t) ---------------
rhs = inline('-2*y + sin(t)', 't', 'y');
h = 0.05;
t = 0:h:2;
y = zeros(size(t));
y(1) = 1;
for i = 1:length(t)-1
    y(i+1) = y(i) + h * feval(rhs, t(i), y(i));
end
% compare with the classical fourth order Runge-Kutta step
yrk = zeros(size(t));
yrk(1) = 1;
for i = 1:length(t)-1
    k1 = feval(rhs, t(i), yrk(i));
    k2 = feval(rhs, t(i) + h/2, yrk(i) + h*k1/2);
    k3 = feval(rhs, t(i) + h/2, yrk(i) + h*k2/2);
    k4 = feval(rhs, t(i) + h, yrk(i) + h*k3);
    yrk(i+1) = yrk(i) + h/6 * (k1 + 2*k2 + 2*k3 + k4);
end
err_euler = max(abs(y - yrk));
fprintf('max difference Euler vs RK4: %e\n', err_euler)

% ---- Section 3.5: ode45 with the old options interface ----------------
opts = odeset('RelTol', 1e-6, 'AbsTol', 1e-8);
[tt, yy] = ode45('j_textbook_rhs', [0 2], 1, opts);
figure(1)
clf
hold on
plot(t, y, 'b-')
plot(t, yrk, 'g--')
plot(tt, yy, 'r.')
hold off
grid
xlabel('t')
ylabel('y(t)')
legend('Euler', 'RK4', 'ode45')
title('Chapter 3 exercise 4')

% ---- Section 3.6: a table of results written to disk --------------------
fid = fopen('ch3_results.txt', 'w');
if fid == -1
    error('could not open results file')
end
fprintf(fid, 'Chapter 3 results, %s\n', date);
fprintf(fid, 'bisection %12.8f\n', root_bisect);
fprintf(fid, 'newton    %12.8f\n', x);
fprintf(fid, 'secant    %12.8f\n', x1);
fclose(fid);

% exercise: keep the number of iterations per method in a vector
iters = [it k length(SECANT_TRACE)];
disp(iters)
if any(iters > 50)
    disp('some method needed too many steps')
end

% a quick check on the random starting points used in the exercise
starts = 1 + 2*rand(5, 1);
for j = 1:length(starts)
    r = fzero(inline('x.^3 - 2*x - 5', 'x'), starts(j));
    fprintf('start %6.3f -> root %10.6f\n', starts(j), r)
end

% pause for the student to look at the figure
% pause
% input('press return to finish ', 's');
