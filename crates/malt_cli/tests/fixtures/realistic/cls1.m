classdef cls1 < handle
    properties (Access = private)
        count = 0
    end
    properties (Constant)
        NAME = "cls1"
    end
    methods
        function obj = cls1()
            obj.count = 0;
        end
        function increment(obj, by)
            arguments
                obj
                by (1,1) double = 1
            end
            obj.count = obj.count + by;
        end
    end
    methods (Static)
        function r = make()
            r = cls1();
        end
    end
end
