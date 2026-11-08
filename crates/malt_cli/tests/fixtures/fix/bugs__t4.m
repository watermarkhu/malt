function t4(a)
x = 1;
y = ++x;
z = --x;
if a == 1 | NaN, end
s = {'a','b'};
if ismember(a, s), end
if isempty(a), end
assert(a);
assert(0);
assert(1 == 1);
h = @() keyboard;
dbstop in t4
cellfun(@isempty, s);
cellfun("isempty", s);
w = who;
