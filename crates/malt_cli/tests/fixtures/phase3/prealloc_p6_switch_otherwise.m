function out = pa2(k, a)
out = zeros(size(a));
switch k
  case 1
    out = a + 1;
  otherwise
    out = a - 1;
end
end
