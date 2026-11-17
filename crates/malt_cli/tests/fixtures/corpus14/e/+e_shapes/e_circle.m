classdef e_circle < d_shape_base
    properties
        Name = 'circle'
        Radius = 1
    end
    methods
        function obj = e_circle(r)
            if nargin
                obj.Radius = r;
            end
        end
        function a = area(obj)
            a = pi * obj.Radius ^ 2;
        end
        function p = perimeter(obj)
            p = 2 * pi * obj.Radius;
        end
        function set.Radius(obj, v)
            if v <= 0
                error('circle:radius', 'Radius must be positive');
            end
            obj.Radius = v;
        end
    end
end
