function y = coder1(u) %#codegen
% Coder-friendly filter
coder.inline('always');
persistent buf idx
if isempty(buf)
    buf = zeros(1, 8);
    idx = 1;
end
buf(idx) = u;
idx = mod(idx, 8) + 1;
y = sum(buf) / numel(buf);
assert(isa(u, 'double'));
end
