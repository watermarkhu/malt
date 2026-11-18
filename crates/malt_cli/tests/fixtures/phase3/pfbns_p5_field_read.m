opts = struct('a', 1);
out = zeros(1, 4);
parfor k = 1:4
    out(k) = opts.a + k;
end
