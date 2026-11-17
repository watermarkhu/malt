function q = fludlrfix(x, s)
%FLUDLRFIX Double flips become one 180-degree rotation.
q = flipud(fliplr(x));
p = fliplr(flipud(s));
q = [q(1), p(1)];
end
