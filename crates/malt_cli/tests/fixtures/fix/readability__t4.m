function t4(x)
if strcmp(class(x), 'char'), end
if isequal(class(x), 'struct'), end
if ischar(x) && strcmp(class(x),'cell'), end
if isa(x, "char"), end
if isa(x, 'struct'), end
if (q = 3) > 2, end
if a = 3, end
while k = 3, end
A = [1, 2,
     3, 4];
B = [1, 2, % c
     3, 4];
C = [1,
     2];
D = [1, 2, ...
     3];
E = [1 2
     3 4];
F = [1 2;
     3 4,
     5 6];
G = {1, 2,
     3, 4};
y = x(1); f = @(t) t(1);
