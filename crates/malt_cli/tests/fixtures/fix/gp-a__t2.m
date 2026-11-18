function r = t2(A, x, ok)
  r = 0;
  if 1 < x < 3, r = 1; end
  v = A(1, end);
  w = A(end, end);
  u = A(end);
  if isempty(A) == true, r = 2; end
  if numel(A) == true, r = 3; end
  if x(1) == true, r = 4; end
  if isempty(A) ~= true, r = 5; end
  if false == isempty(A), r = 6; end
  if true == isempty(A), r = 7; end
  eval('x=1');
  eval(['a' num2str(3) ' = 1']);
  eval('y = windows');
  eval('class.a');
  y = feval(@sin, 1);
  q = [1, % c
       2];
  q2 = [1,% c
       2];
  disp hi, disp there
  for k = 1:3, k = 5; end
  if x, r = 1; end
  b = x + 1;
  if b, r = 1; end
  if 1, r = 1; end
  m = prod(x > 2);
  m = min(x > 2);
  m = max(isx);
  m = max(ok);
  fprintf('%d %d\n', 1);
  fprintf('%d\n', 1, 2);
  fprintf("%d\n", 1, 2);
  fprintf(fid, '%d\n', 1);
  fprintf('%%\n');
  fprintf('%s %*d\n', 'a', 3, 4);
  keyboard
  addpref('a','b',1)
  a = 1:2:3:4;
  if x & ok, r = 1; end
  while x | ok, r = 1; end
  if (x & ok) || r, r = 2; end
end
