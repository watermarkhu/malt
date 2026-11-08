spmd (0.5, 3)
end
spmd (-1, 2)
end
spmd (1, 2, 3)
end
spmd (2)
end
spmd (1, Inf)
end
spmd (n)
end
spmd
  load data
  save foo
  save foo -fromstruct s
  who
  whos -file f.mat
  evalin('caller','x')
  evalin('base','x')
  eval('x=1')
  s = load('x');
  load('x')
  save('f','-fromstruct','s')
  [a,b] = who;
  ans = whos('-file','x');
end
