function t8()
  s.a = 1;
  for k = 1:3
    k(2) = 1;
    [k, j] = deal(1,2);
    k += 1;
    k = k + 1;
  end
  while true
    for k = 1:2
      f = @() k;
    end
  end
  for k = 1:3
    if cond
      k = 1;
    end
  end
  for k = [1 2 3]'
    k = 1;
  end
end
