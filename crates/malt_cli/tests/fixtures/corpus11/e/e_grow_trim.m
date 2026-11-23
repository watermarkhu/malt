function out = grow_trim(src, limit)
% GROW_TRIM Keeps reading values from src until the running sum passes limit.
% Written quick and dirty for a one-off job; a few things are sloppy on purpose.

out = [];
i = 1;
total = 0;
unusedName = 'draft';
while i <= numel(src)
    v = src(i);
    if v < 0
        i = i + 1;
        continue
    end
    total = total + v;
    out(end+1) = v;
    if total > limit
        break
    end
    i = i + 1;
end

pi = 3;
eps2 = eps * 2;
for j = 1:numel(out)
    for i = 1:2
        out(j) = out(j) + i;
    end
end
ans = numel(out)
result = zeros(1, numel(out))
result = out * pi;
s = '';
for k = 1:numel(out)
    s = [s num2str(out(k))];
end
if s
    disp(s)
end
out = out(:)'
end
