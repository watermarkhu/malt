classdef (Abstract) AbsBase < handle
    % ABSBASE An abstract handle class with an abstract property and method.
    properties (Abstract)
        Req
    end
    methods (Abstract)
        out = doit(obj, x)
    end
end
