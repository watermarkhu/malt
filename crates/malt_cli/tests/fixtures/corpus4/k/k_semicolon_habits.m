%% semicolon habits: a script from someone who types ;, and ;; out of habit
% nobody in the team has agreed on a convention yet

a = 5;,
b = a * 2
c = [1 2 3];;
d = c'
e = sum(c)
f = c(2:end);
g = f * 3
h = numel(c);, i1 = h + 1
for ii = 1:3;, disp(ii);, end;
for jj = 1:2
    x = jj ^ 2;;
    y = x + 1
end
k = 0;; while k < 3;, k = k + 1;, end;
if k == 3;, disp('k is three');, end;;
m = magic(3);
m(2, :) = [];
m
n = m(:, 1)';;
o = max(m(:));
p = mean(m)
q = ones(2, 3);, r = zeros(2, 3);
s = q + r
t = 'a string'
u = "a string array"
v = {1, 'two'};;
w = v{2}
z = struct('name', 'demo');
z.count = 4;
z
aa = int8(100) + int8(100)
bb = 1/3;;
cc = round(bb, 2)
dd = true;
ee = ~dd
ff = [];
ff(end+1) = 1;
ff(end+1) = 2
gg = @(x) x.^2;;
hh = gg(4)
ii2 = 10:-2:1
jj2 = linspace(0, 1, 5)
kk = repmat('-', 1, 20)
ll = upper('done');
disp(ll);
mm = fix(-2.5)
nn = rem(10, 4);;
oo = mod(-1, 3)
pp = numel(nn)
%% small helper, defined at the end of the script
function out = add_one(in)
    out = in + 1;
end
qq = add_one(41);
disp(qq)
