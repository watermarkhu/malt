function t1(x, y, s)
try
  a = 1;
end
try, b = 1; catch, end
x = [a + b];
z = [x(1)];
w = [-x];
q = [a]';
r = [a b]';
if s == 'abc', end
if strcmp(lower(s), 'abc'), end
if strcmp(class(x), 'double'), end
if strcmp(s, lower(y)), end
if isequal(class(x), 'double'), end
if 'abc' == 'abcd', end
if 'abc' == 'abc', end
if s ~= 'ab', end
if 'a' ~= 'abc', end
if "abc" == "abcd", end
warning('hello');
warning('a:b', 'hello');
warning("hello");
warning("a:b");
warning('hello: world');
warning(s);
error('hello');
error('a:b', 'x');
error(sprintf('a:b'));
error('Result: %d', 3);
y = -2^2;
y = (-2)^2;
y = -x^2;
y = 2^-2;
y = [a && b];
y = {a || b};
y = [a & b];
y = [(a && b)];
y = x * x';
y = x' * x;
y = (x * x');
y = x * y';
y = x * x' + 1;
y = zeros(numel(x));
y = ones(numel(x), numel(x));
y = zeros(size(x));
y = cell(numel(x));
y = eye(numel(x));
y = nan(numel(x));
y = computer('arch');
y = computer("arch");
y = computer;
y = computer('arch', 'x');
y = a:b:c:d;
end
