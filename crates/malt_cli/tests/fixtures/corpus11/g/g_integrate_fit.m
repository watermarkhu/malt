%% Numerical integration and a constrained fit
f = @(x) exp(-x.^2) .* cos(3*x);
I1 = integral(f, -2, 2);
I2 = quadgk(f, -2, 2, 'RelTol', 1e-8);
I3 = quad(@(x) 1 ./ (1 + x.^2), 0, 1);

opts = optimset('TolX', 1e-10, 'Display', 'off');
root = fzero(@(x) x.^3 - 2*x - 5, 2, opts);
[xmin, fmin] = fminsearch(@(p) sum((p(1) + p(2)*(0:4) - [1 3 5 7 9]).^2), [0 0]);

x = linspace(0, 1, 50)';
y = 2*x + 1 + 0.05*sin(20*x);
A = [ones(size(x)) x];
coef = A \ y;
resid = y - A * coef;
rms = sqrt(mean(resid.^2));
if rms > 0.1
    fprintf('poor fit: rms %.3f\n', rms)
end

lb = [0; 0];
ub = [10; 10];
p0 = [1; 1];
objective = @(p) (p(1) - 3)^2 + (p(2) - 4)^2;
pBest = fmincon(objective, p0, [], [], [], [], lb, ub);

areas = arrayfun(@(a) integral(@(t) t.^a, 0, 1), 1:4);
table_of = [1:4; areas];
disp(table_of)
ratio = areas(2:end) ./ areas(1:end-1)
roots_found = zeros(1, 3);
guesses = [-1, 0.5, 2];
for g = 1:numel(guesses)
    roots_found(g) = fzero(@(x) sin(x) - 0.5*x, guesses(g));
end
final = roots_found(end)
