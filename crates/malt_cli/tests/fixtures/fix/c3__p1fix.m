function p1(n)
global G
persistent Q
s = 0;; t = 1;
parfor i = 1:n
    G = i;
    Q = Q + 1;
    v = G + 1;
    evalin('base', 'x');
    assignin('base', 'x', 1);
    assignin('caller', 'x', 1);
    s = s + i;
end
parfor i = n:1
end
parfor i = 1:n
end
parfor i = 1:n
    s = s + i;
    u = max(u, i);
end
parfor (i = 1:n, 4)
end
parfor i = 1:n, end
parfor i = 1:10
end
for i = 1:2:n
    evalin('base', 'x');
    G = 1;
end
spmd
  evalin('base','x');
  assignin('base','x',1);
  G = 2;
  w = G + Q;
  d = distributed(1:3);
end
spmd(2), G;, end
for k=1:3
    global G
end
end
function f2()
parfor i = 1:5
    evalin("base", "x");
end
end
