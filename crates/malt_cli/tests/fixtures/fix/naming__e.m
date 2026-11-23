classdef Cl
  properties (Access=private)
    pp = 3
    qq (1,1) double = 4
  end
  methods (Abstract)
    absM(obj)
  end
  methods (Static)
    function st()
      cb = @(x) x;
      tt(2).f = 1;
      vv += 1;
      if true, ww = 1; end
    end
  end
end
function helper()
  hh = 1;
end
