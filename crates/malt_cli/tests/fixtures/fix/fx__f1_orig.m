function r = f1(A, x)
  r = 0;
  if numel(A) == true, r = 3; end
  if ~isempty(A) == false, r = 4; end
  if isempty(A) == true && x, r = 5; end
  v = ~(isempty(A) == true);
  z = isempty(A) ~= true == 1;
  q = [1, % c
       2];
  q2 = [1,% c
       2];
  w = strcmp(A, 'a') == false;
  if x(1) == false, r = 6; end
end
