function [theta, pos] = e_robot_arm(l1, l2, target)
%E_ROBOT_ARM Forward and inverse kinematics of a planar 2-link arm.
%   [theta, pos] = e_robot_arm(l1, l2, target) solves the elbow-down
%   inverse kinematics for a target [x y] and returns the joint angles
%   theta = [q1 q2] and the elbow and tip positions pos.
%
%   If the target is out of reach, the arm points straight at it.

if nargin < 3
    target = [1.2, 0.8];
end

reach = l1 + l2;
d = norm(target);
if d > reach
    warning('e_robot_arm:outOfReach', ...
        'Target at distance %.2f is beyond reach %.2f', d, reach);
    theta = [atan2(target(2), target(1)), 0];
    pos = forwardKinematics(l1, l2, theta);
    return
end

% Law of cosines for the elbow angle
c2 = (target(1)^2 + target(2)^2 - l1^2 - l2^2) / (2 * l1 * l2);
c2 = max(min(c2, 1), -1);
q2 = acos(c2);
q1 = atan2(target(2), target(1)) - atan2(l2 * sin(q2), l1 + l2 * cos(q2));
theta = [q1, q2];

pos = forwardKinematics(l1, l2, theta);
end

function pos = forwardKinematics(l1, l2, theta)
% Positions of the elbow and the tip, one row per point.
q1 = theta(:, 1);
q2 = theta(:, 2);
elbow = [l1 * cos(q1), l1 * sin(q1)];
tip = elbow + [l2 * cos(q1 + q2), l2 * sin(q1 + q2)];
pos = struct('elbow', elbow, 'tip', tip);
end

function sweepDemo()
% Vectorised sweep over the workspace, then a quiver of the Jacobian.
l1 = 1.0;
l2 = 0.7;
[q1, q2] = meshgrid(linspace(-pi, pi, 60), linspace(0.05, pi, 40));
x = l1 * cos(q1) + l2 * cos(q1 + q2);
y = l1 * sin(q1) + l2 * sin(q1 + q2);

figure
plot(x(:), y(:), '.', 'MarkerSize', 2)
axis equal
xlabel('x'); ylabel('y')
title('Reachable workspace of a 2-link arm')

% Configurations to test, as a cell array of [x y] targets
targets = {[1.0 0.5], [0.3 1.2], [-0.8 0.9], [2.0 0.0]};
results = cellfun(@(p) e_robot_arm(l1, l2, p), targets, 'UniformOutput', false);

hold on
for k = 1:numel(results)
    tipPos = results{k}.tip;
    plot([0 tipPos(1)], [0 tipPos(2)], '-o')
end
hold off

% Jacobian-based velocity field on a coarse grid
[qa, qb] = meshgrid(-pi:pi/4:pi, 0.2:0.6:pi);
u = -l1 * sin(qa) - l2 * sin(qa + qb);
v = l1 * cos(qa) + l2 * cos(qa + qb);
figure
quiver(qa, qb, u, v, 0.5)
xlabel('q_1 (rad)'); ylabel('q_2 (rad)')
title('Joint-space velocity of the tip for unit q_1 rate')
end
