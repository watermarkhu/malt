psd = 5;
x = psd(3);
function r = foo(psd)
  r = psd(2);
end
function y = inline(a)
  y = a;
end
y = inline(2);
s.psd = 1;
z = s.psd(2);
obj.inline(3);
h = @psd;
cellfun(@inline, {1});
feval('psd', 1);
% psd(x)
str = 'psd(x)';
psd(x);
psd x
a = print(1);
t = tcpclient('a',1);
mupad
