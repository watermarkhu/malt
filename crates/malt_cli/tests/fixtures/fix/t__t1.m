for i=1:3
  x = {1,2};
  y = x{end};
  z = x(end);
  w = 1:end;
  if i == 2, break, end
end
break
continue
a = [1 2 3];
b = a(min(end, 2));
c = a(end+1);
f = @() a(end);
g = end;
while true, break; end
switch 1
  case 1
    break
end
try, error('A:b'); catch, end
error('A:b');
error("A:b");
error('Result: done');
error('Hello: world %d', 3);
warning('A:b');
warning('on','all');
warning('off:all');
disp('x: y')
