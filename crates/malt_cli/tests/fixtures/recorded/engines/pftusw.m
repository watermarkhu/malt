function r = f(n)
r = zeros(1, n);
parfor i = 1:n
    r = i;
end
end
