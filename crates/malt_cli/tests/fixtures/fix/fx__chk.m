function r = f1(A, x)
  r = 0;
;;  if numel(A), r = 3; end
;;  if ~isempty(A) == false, r = 4; end
;;  if isempty(A) && x, r = 5; end
;;  v = ~(isempty(A));
  z = ~isempty(A) == 1;
  q = [1; % c
       2];
  q2 = [1;% c
       2];
  w = ~strcmp(A, 'a');
;;  if ~x(1), r = 6; end
end
