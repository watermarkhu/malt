classdef Point < handle
    % A point in the plane.
    properties
        X double = 0
        Y double = 0
    end
    methods
        function obj = Point(x, y)
            obj.X = x;
            obj.Y = y;
        end
        function d = norm(obj)
            d = hypot(obj.X, obj.Y);
        end
    end
end
