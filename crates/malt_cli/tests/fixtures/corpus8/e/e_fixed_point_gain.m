function y = e_fixed_point_gain(u, k)
%#codegen
%R4_FIXED_POINT_GAIN Saturating gain with a one-sample delay, codegen style.
%   Y = R4_FIXED_POINT_GAIN(U, K) returns the saturated, delayed gain of U.

coder.inline('never');
persistent prev
if isempty(prev)
    prev = zeros(1, 3);
end

y = zeros(size(u));
for i = 1:numel(u)
    v = k * u(i);
    if v > 10
        v = 10;
    elseif v < -10
        v = -10;
    end
    y(i) = prev(1);
    prev = [v, prev(1:2)];
end

table = zeros(1, 8);
for j = 1:8
    table(j) = 2^(j - 1);
end
acc = 0;
for j = 1:numel(table)
    acc = acc + table(j);
end
scaled = double(int16(y * 100)) / 100;
out = single(scaled);
n = numel(out);
buffer = zeros(1, n);
buffer(1:n) = out;
end
