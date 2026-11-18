classdef Sub3 < Base & handle
    methods
        function obj = Sub3(x)
            if nargin == 0
                x = 1;
            end
            obj = obj@Base(x);
            obj@handle();
        end
        function bar(obj)
            foo@Base(obj);
            baz@Base(obj);
        end
    end
end
