function r = n_lbodup_elementwise(a, x)
% Identical operands: reported for the short-circuit operators only.
r = [a && a, a || a, a & a, a | a, x(1) & x(1), 1 & 1];
end
