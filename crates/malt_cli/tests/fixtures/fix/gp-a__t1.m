x = 1;
if 1 < x < 3, disp(1); end
s.a.b.c.d(1) = 2;
y = s.a.b.c.d(3);
try
  x = 1;
catch
  disp('e');
end
try
  x = 2;
catch ME
  warning(ME);
  warning(ME.message);
end
eval('load(filename)');
eval('x = 1');
eval(['load ' 'f']);
eval('s.a = 1');
eval('ls');
load('data.mat');
load data.mat
q = load('data.mat');
thisisaveryveryveryverylongvariablenamethatexceeds63charactersxxxxxxxxxxxxxxxxxx = 1;
%{ 
load('x')
%}
disp('eval(''x=1'')');
