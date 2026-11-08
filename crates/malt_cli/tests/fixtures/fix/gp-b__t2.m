function t2(c, s, a, b)
c{:}
myfun(1)
x = 1;
ans
y = ans + 1;
ans = 5;
disp(ans)
y = [c{:}];
y = [s.f];
y = [c{1}];
y = [a(:)'];
y = [a]';
y = ['a'];
y = [];
y = [a,];
y = [ a ];
y = [a  % comment
];
y = [a; ];
z = [a]' * 2;
y = [1:3];
y = [a(1:3)];
y = [@sin];
y = [a.b];
y = [{1}];
y = [~a];
y = [a++b];
warning('off', 'all');
warning('on', 'MATLAB:foo');
warning off all
warning(a.identifier, '%s', a.message);
warning(s);
error(s);
error('Result: %d', 3);
error('MyPkg:id', 'msg %d', 3);
error(struct('message','x','identifier','a:b'));
error('msg:with spaces');
if s == 'a', end
if 'a' == s, end
if s == "abc", end
if s ~= 'abc', end
if s == 'abc' && true, end
v = s == 'ab';
if strcmp(upper(s), 'ABC'), end
if strcmp(s, upper(a)), end
if strncmp(lower(s), 'a', 1), end
if strcmp(class(a), class(b)), end
if strcmp(a.class, 'x'), end
if class(a) == 'double', end
if strcmp(class(a), 'double') || strcmpi(class(b),'single'), end
end
