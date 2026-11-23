classdef (Abstract) d_shape_base
    properties (Abstract)
        Name
    end
    properties (Access = protected)
        Tag = ''
    end
    methods (Abstract)
        a = area(obj)
        p = perimeter(obj)
    end
    methods
        function describe(obj)
            fprintf('%s: area=%g perimeter=%g\n', obj.Name, obj.area(), obj.perimeter());
        end
        function r = ratio(obj)
            r = obj.area() / obj.perimeter();
        end
        function tf = lt(a, b)
            tf = a.area() < b.area();
        end
    end
end
