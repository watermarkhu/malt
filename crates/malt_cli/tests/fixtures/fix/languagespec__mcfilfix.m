classdef OldShape
    %OLDSHAPE A class whose declaration must follow its file name.
    properties
        Value (1,1) double
    end
    methods
        function obj = OldShape(v)
            obj.Value = v;
        end
        function out = twice(obj)
            out = obj.Value * 2;
        end
    end
end
