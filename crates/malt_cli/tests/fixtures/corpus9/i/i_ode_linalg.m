%% Forced spring-mass-damper: ODE solve, FFT check and a least-squares fit
clear all
close all

m = 2.0; c = 0.4; k = 5.0;
F = @(t) 3*sin(1.3*t) + 0.5*(t > 10);
rhs = @(t, y) [y(2); (F(t) - c*y(2) - k*y(1)) / m];

[t, y] = ode45(rhs, [0 30], [0; 0]);
pos = y(:, 1);
vel = y(:, 2);

% Sample on a uniform grid for the FFT
tu = linspace(0, 30, 2048);
pu = interp1(t, pos, tu, 'linear');
P = abs(fft(pu - mean(pu)));
f = (0:numel(P)-1) * (1/(tu(2)-tu(1))) / numel(P);
[~, peak] = max(P(2:floor(end/2)));
dominant = f(peak + 1);

% Least-squares estimate of the stiffness from the forced response
A = [pos, vel];
b = m * gradient(vel, t) + c * vel;
x = A \ b;
Ainv = inv(A' * A);
coef = Ainv * (A' * b);

energy = 0.5*m*vel.^2 + 0.5*k*pos.^2;
bigEnergy = find(energy > 5);
firstBig = bigEnergy(1)
peakPos = max(pos(pos > 0))

mask = pos > 0.2 & vel < 0;
crossings = sum(diff(mask) == 1)
clipped = pos;
clipped(clipped > 1) = 1;
clipped(clipped < -1) = -1;

result = struct('dominantHz', dominant, 'k', x(1));
fprintf('estimated stiffness %.3f, dominant %.3f Hz\n', result.k, result.dominantHz);
residual = norm(A*x - b)
cond_num = cond(A);
tail = y(end-10:end, :);
mean_tail = mean(tail)
