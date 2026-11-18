function g_robot_sim(nsteps, seedval)
% G_ROBOT_SIM  toy 2D robot random walk with a wall
% hobby project, made in the evenings.
% globals POS, VEL, LOGBUF are shared between the functions in this file

global POS VEL LOGBUF
if nargin < 1, nsteps = 500; end
if nargin < 2, seedval = 1; end

rand('seed', seedval);
randn('state', seedval);
POS = [0 0];
VEL = [0 0];
LOGBUF = [];

figure
hold on
for t = 1:nsteps
    step_robot(0.05);
    log_event(t);
    if mod(t, 10) == 0
        plot(POS(1), POS(2), 'r.')
        drawnow
    end
end
hold off
fprintf('final position: %f %f\n', POS(1), POS(2))
fprintf('events logged: %d\n', numel(LOGBUF));

% random obstacles, one per loop, magic numbers
obs = [];
for k = 1:20
    obs = [obs; rand*10-5 rand*10-5];
end
plot(obs(:,1), obs(:,2), 'ks')

% pause for the animation the prof wanted
for t = 1:50
    POS = POS + [0.01 0.01];
    plot(POS(1), POS(2), 'b.')
    pause(0.01)
end

% energy-ish number, not used anywhere
x = rand(1, 100);
m = mean(x);
v = var(x);
y = randn(10, 1);

% old version of the step, commented
%   VEL = VEL + randn(1,2);
%   POS = POS + VEL;
end

function step_robot(dt)
% one time step, bounce off the wall at |x| = 5
global POS VEL LOGBUF
noise = randn(1,2) * 0.2;
VEL = VEL + noise*dt;
if abs(POS(1)+VEL(1)*dt) > 5
    VEL(1) = -VEL(1);
end
POS = POS + VEL*dt;
end

function log_event(t)
% logs every 25th step, counter kept between calls
persistent count
global LOGBUF POS
if isempty(count)
    count = 0;
end
if mod(t, 25) == 0
    count = count + 1;
    LOGBUF(end+1) = count;
end
if POS(1) > 4 | POS(1) < -4
    disp('near wall')
end
end
