function r = pf_temp_read_before_set(n)
u = 0;
r = zeros(1, n);
parfor k = 1:n
    w = u;
    u = k;
    r(k) = w;
end
end
