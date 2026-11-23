function c = minv_p4_09(A, b)
M = inv(A);
if b(1) > 0
    c = M*b;
else
    c = b;
end
end
