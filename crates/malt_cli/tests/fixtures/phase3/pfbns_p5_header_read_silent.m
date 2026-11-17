v = rand(1, 10);
out = zeros(1, 10);
parfor k = 1:numel(v)
    out(k) = v(k) * 2;
end
