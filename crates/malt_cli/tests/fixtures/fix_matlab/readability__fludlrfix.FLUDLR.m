function q = fludlrfix(x, s)
%FLUDLRFIX Double flips become one 180-degree rotation.
q = rot90(x,2);
p = rot90(s,2);
q = [q(1), p(1)];
end
