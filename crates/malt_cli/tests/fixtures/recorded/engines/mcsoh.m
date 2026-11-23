classdef Foo < handle
    properties
        X
    end
    methods
        function obj = set.X(obj, val)
            obj.X = val;
        end
    end
end
