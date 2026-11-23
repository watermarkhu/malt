global G H
G = 1;
function f()
  global G
  G = 2;
  x = G;
end
function g()
  y = G;
end
