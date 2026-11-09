%% Setup
x = linspace(0, 1, 100);
y = sin(2*pi*x);
A = magic(4);
A(2, :) = [];
B = A';
c = {1, 'two', [3 4]};
for i = 1:numel(c)
    item = c{i};
    if ischar(item)
        fprintf('%s\n', item);
    end
end
names = fieldnames(struct('a', 1, 'b', 2));
t = tic;
while toc(t) < 0.01
end
z = x(end:-1:1);
m = containers.Map();
m('k') = 1;
v = m('k');
result = cellfun(@(q) numel(q), c);
fprintf('%d\n', result);
s = sprintf('%d', v);
disp(s)
