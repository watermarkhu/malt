function out = pa10(k, a)
out = zeros(size(a));
try
  out = a + k;
catch
  out = 0;
end
end
