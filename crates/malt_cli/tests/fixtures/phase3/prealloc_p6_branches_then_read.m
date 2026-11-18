function pa7(k, a)
out = zeros(size(a));
if k == 1
  out = a + 1;
else
  out = a - 1;
end
disp(out)
end
