function r = pf_indexed_input_later(n)
a = zeros(1, n);
parfor k = 1:n
    a(k) = v(k);
end
v = 1:n;
r = a;
end
