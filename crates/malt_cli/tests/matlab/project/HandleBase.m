classdef HandleBase < handle
    % HANDLEBASE A handle class with a counter, a static method and a method.
    properties
        Count = 0
    end
    methods
        function obj = HandleBase()
        end
        function bump(obj, n)
            obj.Count = obj.Count + n;
        end
    end
    methods (Static)
        function r = zero()
            r = 0;
        end
    end
end
