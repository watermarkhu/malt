function k_nbrak_field(obj, v, n)
% A single indexed field in brackets is a scalar expression.
r1 = [obj.Value( end ) ]* n;
r2 = [obj.Value(end)];
r3 = [obj.names] * n;
r4 = [v(end)] * n;
disp(r1 + r2 + r3 + r4);
end
