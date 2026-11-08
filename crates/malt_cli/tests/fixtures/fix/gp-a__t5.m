function t5()
  helper(1);
  helper(1,2,3);
  helper();
  y = max(1,2,3,4);
  z = numel();
  s = sin(1,2);
  w = foo(1,2);
end
function helper(a,b)
  disp(a+b)
end
function foo(varargin)
end
