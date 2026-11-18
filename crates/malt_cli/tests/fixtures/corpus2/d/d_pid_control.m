%% PID loop with a discrete first-order plant
% Simulates a PID controller tracking a step setpoint on a
% discrete-time first-order plant with a transport delay.
%
% Plant:   y[k+1] = a*y[k] + b*u[k-d]
% Control: PID with derivative on measurement, output saturated.

clear; clc; close all;

%% Parameters
Ts = 0.05;              % sample time [s]
tEnd = 20;              % simulation horizon [s]
t = 0:Ts:tEnd;
N = numel(t);

a = 0.92;               % plant pole
b = 0.08;               % plant gain
delay = 3;              % transport delay in samples

Kp = 2.4;
Ki = 1.1;
Kd = 0.15;
uMin = -5;
uMax = 5;

%% Setpoint: step up at t = 1 s, then step down at t = 12 s
r = zeros(1, N);
r(t >= 1) = 1.0;
r(t >= 12) = 0.4;

%% Preallocate signals
y = zeros(1, N);        % plant output
u = zeros(1, N);        % controller output
e = zeros(1, N);        % tracking error
uP = zeros(1, N);       % proportional term
uI = zeros(1, N);       % integral term
uD = zeros(1, N);       % derivative term

integrator = 0;
prevY = 0;

%% Main loop
for k = 2:N
    e(k) = r(k) - y(k-1);

    uP(k) = Kp * e(k);

    % Integrate with simple anti-windup: only accumulate when unsaturated
    if u(k-1) > uMin && u(k-1) < uMax
        integrator = integrator + Ki * e(k) * Ts;
    end
    uI(k) = integrator;

    % Derivative on measurement to avoid setpoint kick
    uD(k) = -Kd * (y(k-1) - prevY) / Ts;
    prevY = y(k-1);

    uRaw = uP(k) + uI(k) + uD(k);
    u(k) = min(max(uRaw, uMin), uMax);

    % Plant update with transport delay
    if k - delay >= 1
        uDelayed = u(k - delay);
    else
        uDelayed = 0;
    end
    y(k) = a * y(k-1) + b * uDelayed;
end

%% Performance metrics
firstStep = t >= 1 & t < 12;
overshoot = max(0, (max(y(firstStep)) - 1.0) / 1.0) * 100;
fprintf('Overshoot (first step): %.1f %%\n', overshoot);
fprintf('Max |control|: %.2f\n', max(abs(u)));
fprintf('Mean abs error: %.4f\n', mean(abs(e(2:end))));

%% Plots
figure('Name', 'PID loop', 'Color', 'w');

subplot(3, 1, 1);
plot(t, r, 'k--', t, y, 'b', 'LineWidth', 1.2);
ylabel('Output');
legend('Setpoint', 'Plant output', 'Location', 'southeast');
grid on;
title('Closed-loop response');

subplot(3, 1, 2);
plot(t, u, 'r');
ylabel('Control u');
yline(uMax, ':');
yline(uMin, ':');
grid on;

subplot(3, 1, 3);
plot(t, uP, t, uI, t, uD);
xlabel('Time [s]');
ylabel('Terms');
legend('P', 'I', 'D');
grid on;
