classdef Square < shapes.Shape
    % Square has four equal sides.

    properties
        Side (1,1) double = 1
    end

    methods
        function obj = Square(side)
            obj@shapes.Shape("square");
            obj.Side = side;
        end

        function a = area(obj)
            a = obj.Side^2;
        end
    end
end
