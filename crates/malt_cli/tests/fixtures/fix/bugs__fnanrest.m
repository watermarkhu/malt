function r = fnanrest(n, m)
%FNANREST A NaN comparison with an elementwise operator behind it.
r = 0;
if n == NaN | m > 3
    r = 1;
end
end
