function r = pf_inner_loop_subscript(n)
y = zeros(1, 3);
parfor i = 1:n
    for j = 1:3
        y(j) = i;
    end
end
r = y;
end
