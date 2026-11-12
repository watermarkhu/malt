classdef l_rectangle < l_shape
    %L_RECTANGLE Axis-aligned rectangle with a length unit.
    %   The unit is an l_unit enumeration member and is carried along
    %   with the geometry, but it does not change the numeric area.

    properties
        Width (1,1) double {mustBeNonnegative} = 1
        Height (1,1) double {mustBeNonnegative} = 1
        Unit l_unit = l_unit.metre
    end

    methods
        function obj = l_rectangle(w, h, unit)
            %L_RECTANGLE Construct a rectangle.
            arguments
                w (1,1) double = 1
                h (1,1) double = 1
                unit l_unit = l_unit.metre
            end
            obj@l_shape('rectangle');
            obj.Width = w;
            obj.Height = h;
            obj.Unit = unit;
        end

        function a = area(obj)
            a = obj.Width * obj.Height;
        end

        function p = perimeter(obj)
            p = 2 * (obj.Width + obj.Height);
        end

        function s = describe(obj)
            %DESCRIBE Extend the template summary with the unit symbol.
            s = sprintf('%s [%s]', describe@l_shape(obj), obj.Unit.Symbol);
        end

        function r = rotated(obj)
            %ROTATED Return a copy with width and height swapped.
            r = l_rectangle(obj.Height, obj.Width, obj.Unit);
        end

        function c = toCircle(obj)
            %TOCIRCLE Circle with the same area, centred on the origin.
            c = l_circle(sqrt(obj.Area / pi));
        end

        function setUnit(obj, unit)
            %SETUNIT Change the unit of the rectangle.
            obj.Unit = unit;
            obj.notifyChanged();
        end
    end

    methods (Static)
        function r = square(side, unit)
            %SQUARE Construct a rectangle with equal sides.
            if nargin < 2
                unit = l_unit.metre;
            end
            r = l_rectangle(side, side, unit);
        end
    end

    methods (Access = private)
        function legacy = oldArea(obj, unused)
            legacy = obj.Width * obj.Height;
        end
    end
end
