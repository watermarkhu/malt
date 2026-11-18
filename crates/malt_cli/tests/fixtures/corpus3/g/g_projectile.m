function [xs, ys, rng, tmax] = g_projectile(v0, angle_deg, dt)
% G_PROJECTILE simulate projectile with quadratic drag
%   [xs, ys, rng, tmax] = g_projectile(v0, angle_deg, dt)
%   Euler integration. Drag coefficient is hard coded
%   m = 0.145 kg (baseball-ish), rho = 1.2
%
% old version used ode45 but Euler is fine for now -- ask Ana

if nargin < 1
    v0 = 40;
end
if nargin < 2, angle_deg = 35; end
if nargin < 3
    dt = 0.001;
end

g = 9.8;
m = 0.145;
Cd = 0.4;
A = 0.0042;
rho = 1.2;
k = 0.5 * rho * Cd * A / m;

th = angle_deg * pi / 180;
vx = v0 * cos(th);
vy = v0 * sin(th);
x = 0; y = 0; t = 0;
xs = [x]; ys = [y];
tmax = 0;
while y >= 0
    v = sqrt(vx^2 + vy^2);
    ax = -k * v * vx;
    ay = -g - k * v * vy;
    vx = vx + ax*dt;
    vy = vy + ay*dt;
    x = x + vx*dt;
    y = y + vy*dt;
    t = t + dt;
    xs(end+1) = x;
    ys(end+1) = y;
    if t > 100
        break;
    end
end
rng = x;
tmax = t;

% plot(xs, ys); % old plot
% hold on

% energy check, never used in the report
E = 0.5*m*(vx^2+vy^2) + m*g*y;

end

function g_projectile_driver()
% driver: sweep angles and velocities, plot the ranges
angles = [20 30 45 60];
res = zeros(1, length(angles));
for i = 1:length(angles)
    [xs, ys, rng, tmax] = g_projectile(40, angles(i), 0.001);
    res(i) = rng;
    fprintf('angle %d range %.2f time %.2f\n', angles(i), rng, tmax);
end
best = max(res);
idx = find(res == best);
disp(['best angle: ' num2str(angles(idx))])

% velocity sweep, copy of the loop above with a different variable
vels = 10:10:60;
rv = [];
for i = 1:length(vels)
    [xs, ys, rng, tmax] = g_projectile(vels(i), 45, 0.001);
    rv(end+1) = rng;
end
rv

% same thing again with dt changed, to see if it matters
for j = 1:length(vels)
    [xs2, ys2, rng2, tmax2] = g_projectile(vels(j), 45, 0.01);
    rv2(j) = rng2;
end
rv2

figure
plot(angles, res, 'o-')
xlabel('angle'); ylabel('range (m)')
title(sprintf('range vs angle, v0=%d', 40))

figure
plot(vels, rv, 'x-')
hold on
plot(vels, rv2, 'r.-')
hold off
xlabel('v0'); ylabel('range')
legend('dt=0.001', 'dt=0.01')

% 'i' used as a plain variable too
i = sqrt(-1);
disp(i)
end
