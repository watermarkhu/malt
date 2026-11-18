% Analyze data
clear; clc;
data = load('data.mat');
n = numel(data.x);
result = zeros(1, n);
for k = 1:n
    result(k) = data.x(k)^2 + sin(k);
end
figure;
plot(result, 'LineWidth', 2);
title('Squares');
if any(result > 10) && ~isempty(result)
    fprintf('Max: %f\n', max(result));
end
names = {'a', 'b'};
for i = 1:numel(names)
    s.(names{i}) = i;
end
disp(s);
