a = repmat(true, 3, 4);
b = repmat(false, 3, 4);
c = repmat(Inf, 3, 4);
d = repmat(NaN, 3, 4);
disp(a + b + c + d);
