function [a, b] = c_fxseta_multiout(n)
% A loop variable used as an index of an output is only read.
a = zeros(1, n);
b = cell(1, n);
for k = 1:n
    [a(k), b{k}] = deal(k, num2str(k));
end
for j = 1:n
    [a, j] = deal(1, 2);
end
end
