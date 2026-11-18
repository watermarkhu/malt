function g_ode_driver
% damped oscillator with event detection
mu = 0.3;
omega = 2;
f = @(t, y) [y(2); -omega^2 * y(1) - mu * y(2)];
opts = odeset('RelTol', 1e-8, 'AbsTol', 1e-10, 'Events', @zero_cross);
[t, y, te, ye, ie] = ode45(f, [0 20], [1; 0], opts);
fprintf('%d steps, %d events\n', length(t), length(te));
amp = max(abs(y(:, 1)));
r = fzero(@(x) cos(x) - x, [0 1]);
o = optimset('Display', 'off', 'TolX', 1e-9);
[xmin, fval, flag] = fminsearch(@(p) sum((p(1) * exp(-p(2) * t) - y(:, 1)).^2), [1 0.1], o);
if flag ~= 1
    warning('did not converge');
end
tt = linspace(0, 20, 400);
yy = interp1(t, y(:, 1), tt, 'spline');
plot(tt, yy, 'LineWidth', 1.5), hold on
plot(te, ye(:, 1), 'rx')
legend('x(t)', 'zero crossings')
xlabel('t'); ylabel('x');
grid on
    function [value, isterminal, direction] = zero_cross(~, yv)
        value = yv(1);
        isterminal = 0;
        direction = 0;
    end
end
