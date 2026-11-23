function t1(x, y)
while true
    x = x + 1;
    y = y - 1;
;    if x > 3, break; end
end
if isnan(x), disp(1); end
if ~isnan(x), disp(1); end
if x && y, disp(2); end
if x || y, disp(2); end
if ~any(x == 1, "all"), disp(3); end
r = strcmp(x, {'a','b'});
q = strcmp('abc','ABC');
z = zeros(3)*2;
w = zeros(3) && y;
v = rand(3) || y;
try
    error('a');
catch
    disp('x');
end
try
    error('a');
catch e
    warning(e);
end
elseif x, a=2; end
elseif y, a=1; else a=1; end
switch x
    case 1
    a=1;
    case 1
    a=2;
    case NaN
    a=3;
end
c = a || a;
d = x(1) | y;
assert('msg');
assert(true, 'msg');
keyboard;
feval('sin', 1);
cellfun('isempty', {1});
[a,b] = 5;
[a,b] = x;
