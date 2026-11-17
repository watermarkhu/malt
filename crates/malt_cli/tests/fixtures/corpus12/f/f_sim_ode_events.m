function [tOut, yOut, info] = sim_ode_events(params)
%SIM_ODE_EVENTS Bouncing ball with an event that stops at the first impact.

if nargin < 1
    params = struct('g', 9.81, 'e', 0.8, 'tEnd', 10);
end
g = params.g;
e = params.e;

opts = odeset('Events', @impact, 'RelTol', 1e-6, 'AbsTol', 1e-8);
y0 = [10; 0];
tOut = [];
yOut = [];
tStart = 0;
info = struct('bounces', 0, 'times', []);
while tStart < params.tEnd && info.bounces < 5
    [t, y, te, ye] = ode45(@ballRhs, [tStart params.tEnd], y0, opts);
    tOut = [tOut; t];
    yOut = [yOut; y];
    if isempty(te)
        break
    end
    info.bounces = info.bounces + 1;
    info.times(end+1) = te(end);
    y0 = [0; -e * ye(end, 2)];
    tStart = te(end);
end
energy = 0.5 * yOut(:, 2).^2 + g * yOut(:, 1);
peakEnergy = max(energy)
done = true;

    function dy = ballRhs(~, y)
        dy = [y(2); -g];
    end

    function [value, isterminal, direction] = impact(~, y)
        value = y(1);
        isterminal = 1;
        direction = -1;
    end
end
