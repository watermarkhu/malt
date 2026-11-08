function t10(i)
  eval('x = windows');
  eval(['x' num2str(i) ' = 0']);
  eval(['x' num2str(i)]);
  m = max(island);
  m = min(isize);
  m = max(hash);
  m = max(has_nan);
  m = prod(~x);
  m = max(a > b, [], 2);
  A = magic(3);
  B = A(end, 1);
  C = A(1, end);
  D = A(2:end, end);
  E = f(x)(end);
  g = A(end);
  fprintf('%d %d\n', [1 2]);
  fprintf('%s: %d\n', name);
  s = sprintf('%d%%', 5);
  s = sprintf('%5.2f %s', 1.5);
  s = sprintf('abc');
  s = sprintf(fmt, 1, 2);
  s = sprintf('%d', 1, 2, 3);
  keyboard;
  disp('keyboard');
  keyboard = 3;
  a = 1:2:3:4:5;
  b = x(1:2:3:4);
  disp a, disp b
  disp a; disp b
  x = 5; % comment, with comma
  c = {1, % hi
       2};
  % [1, % not comment
  d = "[1, % in string";
  e = '[1, % in char';
end
