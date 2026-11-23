function out = pb6(n)
out = zeros(1, n);
if n > 1
  out = 1;
else
  out = 2;
end
out(end+1) = 3;
end
