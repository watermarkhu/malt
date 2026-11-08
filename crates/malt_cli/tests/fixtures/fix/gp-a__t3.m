function r = t3(x, ok)
  r = 0;
  if x & ok, r = 1; end
  if (x & ok), r = 1; end
  if x & ok || r, r = 1; end
  if a(1) == 1 & ok, r = 3; end
end
