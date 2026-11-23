function i_legacy_sloppy(a, b, c)
% R3_LEGACY_SLOPPY  A sloppy old routine: unused arguments, growing strings,
% and a few statements that print by accident.

result = 0
msg = ''
for i = 1:10
    msg = [msg num2str(i) ' ']; %#ok<AGROW>
    result = result + i
end

[~, idx] = max([3 9 2]);
[mx, ~] = min([4 1 7]);

if a == 1
    b = 2;
end
flag = b > 1;

list = [];
k = 1;
while k < 5
    list(k) = k^2;
    k = k + 1;
end
list

%% Section two, no real work here
%% just a cell marker

x = 5;;
y = x';
z = y(:)'

v = 1:3;
w = v + 1;
clear v

names = {'a', 'b'};
for n = names
    disp(n{1})
end

str = "text";
t = ['abc' str];
q = strcat('x ', 'y');

global CONFIG_LEVEL
CONFIG_LEVEL = 3;
s.level = CONFIG_LEVEL;
s.name = 'foo';
s.level = s.level + 1
end
