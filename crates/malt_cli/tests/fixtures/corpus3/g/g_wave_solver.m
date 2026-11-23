% 2D wave equation, explicit FD. messing with this for the numerics course
clear all; close all; clc
c = 1.0;
L = 1; N = 60;
dx = L/N; dt = 0.5*dx/c;
T = 2;
nsteps = round(T/dt);
x = linspace(0, L, N+1);
y = linspace(0, L, N+1);
[X, Y] = meshgrid(x, y);
u = zeros(N+1, N+1);
uold = zeros(N+1, N+1);
unew = zeros(N+1, N+1);

% initial bump
for i = 1:N+1
    for j = 1:N+1
        r2 = (X(i,j)-0.5)^2 + (Y(i,j)-0.5)^2;
        u(i,j) = exp(-50*r2);
    end
end
uold = u;
ncheck = 0;
Emax = [];

for n = 1:nsteps
    for i = 2:N
        for j = 2:N
            unew(i,j) = 2*u(i,j) - uold(i,j) + (c*dt/dx)^2 * (u(i+1,j) + u(i-1,j) + u(i,j+1) + u(i,j-1) - 4*u(i,j));
        end
    end
    % boundaries fixed at zero
    unew(1,:) = 0; unew(end,:) = 0; unew(:,1) = 0; unew(:,end) = 0;
    uold = u;
    u = unew;
    if mod(n, 5) == 0
        surf(X, Y, u)
        shading interp
        zlim([-1 1])
        pause(0.01)
        ncheck = ncheck + 1;
    end
    Emax(n) = max(max(abs(u)));
end
figure
plot(Emax)
xlabel('step'), ylabel('max |u|')

% same solver again with c = 0.5, copy and paste from above
c = 0.5;
dt = 0.5*dx/c;
nsteps2 = round(T/dt);
u2 = zeros(N+1, N+1);
uold2 = u2;
for n = 1:nsteps2
    unew2 = zeros(N+1, N+1);
    for i = 2:N
        for j = 2:N
            unew2(i,j) = 2*u2(i,j) - uold2(i,j) + (c*dt/dx)^2 * (u2(i+1,j) + u2(i-1,j) + u2(i,j+1) + u2(i,j-1) - 4*u2(i,j));
        end
    end
    uold2 = u2;
    u2 = unew2;
end

% 1D cut through the middle, with hold on
figure
mid = round(N/2) + 1;
plot(x, u(mid,:), 'b')
hold on
plot(x, u2(mid,:), 'r--')
hold off
legend('c=1', 'c=0.5')

% mesh of the final state
figure
mesh(X, Y, u)
hold on
plot3(X(mid,mid), Y(mid,mid), u(mid,mid), 'ko')
hold off
colorbar

% energy, sloppy
kinetic = 0.5*sum(sum((u - uold).^2)) / dt^2;
potential = 0.5*c^2*sum(sum(u.^2));
E = kinetic + potential
fprintf('energy %g, max %g\n', E, max(max(u)));

% zero matrix sized after the loop, unused
Ztmp = zeros(size(u));
Ztmp(1,1) = 1;

% old version with a while loop, left in on purpose
% t = 0;
% while t < T
%     t = t + dt;
% end

% check stability
CFL = c*dt/dx
if CFL > 1
    disp('unstable, reduce dt')
end
i = 1;
ii = 2;
j = i + ii
