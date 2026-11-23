parfor i = 1:2:10
    a(i) = i;
end
parfor i = 10:-1:1
    b(i) = i;
end
parfor ans = 1:3
end
parfor i = 1:1.0:5
end
n = 5;
parfor i = 1:n
  c(i) = i;
end
parfor i = n:-1:1
end
parfor (i = 1:10, 2)
  d(i) = 1;
end
parfor i = 1:3
  load data
  save foo
  save('foo','-fromstruct','s')
  save foo -fromstruct s
  x = nargin;
  y = nargin('f');
  s = load('f');
  load('f');
  s = who;
  who
end
