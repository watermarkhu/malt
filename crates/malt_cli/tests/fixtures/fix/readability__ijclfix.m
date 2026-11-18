function z = ijclfix(u)
%IJCLFIX Bare i and j as the imaginary unit rewrite to 1i.
z = u + j;
w = 3 * i;
z = z + w;
end
