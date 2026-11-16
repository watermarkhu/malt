classdef HandSub < HandleBase & AbsBase
    % HANDSUB A handle class that inherits a handle class and an abstract one.
    properties
        Req
    end
    methods
        function out = doit(obj, x)
            out = x + obj.Count;
        end
    end
end
