function out = i_broken_if(x, scale)
% R2_BROKEN_IF  Deliberately broken: the first if has no end.
%   Used to see how malt reports a missing end in the middle of a file.

if nargin < 2
    scale = 1;
end

out = zeros(size(x));
for i = 1:numel(x)
    out(i) = x(i) * scale;
end

total = sum(out(:));
mu = mean(out(:));
sigma = std(out(:));

labels = cell(size(x));
for i = 1:numel(x)
    if x(i) > mu
        labels{i} = 'above';
    elseif x(i) < mu
        labels{i} = 'below';
    else
        labels{i} = 'equal';
    end
end

counts = struct('above', 0, 'below', 0, 'equal', 0);
for i = 1:numel(labels)
    counts.(labels{i}) = counts.(labels{i}) + 1;
end

z = (out - mu) / max(sigma, eps);
clipped = min(max(z, -3), 3);

if total > 100
    out = out / total;

for k = 1:numel(clipped)
    out(k) = out(k) + clipped(k);
end

while max(out) > 10
    out = out / 2;
end

final = sum(out);
end
