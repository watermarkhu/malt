classdef Shape < handle
    % Shape is the base of the planar shapes.

    properties
        % Name of the shape.
        Name (1,1) string = "shape"
        % Number of sides.
        Sides (1,1) double = 0
    end

    methods
        function obj = Shape(name)
            % Create a shape with a name.
            obj.Name = name;
        end

        function a = area(obj)
            % Area of the shape.
            a = 0;
        end
    end

    methods (Static)
        function s = unit()
            s = shapes.Shape("unit");
        end
    end
end
