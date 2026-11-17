function c = minv_p4_10(A, b)
M = pinv(A);
c = M*b;
end
