classdef a_point_value
    %A_POINT_VALUE Two-dimensional point with simple arithmetic.
    %   P = A_POINT_VALUE(X, Y) creates a point. Operators act
    %   component-wise, and EQ compares within a tolerance.

    properties
        X (1,1) double {mustBeFinite} = 0
        Y (1,1) double {mustBeFinite} = 0
    end

    properties (Constant)
        Tolerance (1,1) double = 1e-9
    end

    properties (Dependent)
        Norm (1,1) double
    end

    methods
        function obj = a_point_value(x, y)
            %A_POINT_VALUE Constructor.
            if nargin == 2
                obj.X = x;
                obj.Y = y;
            end
        end

        function r = plus(a, b)
            %PLUS Component-wise sum.
            r = a_point_value(a.X + b.X, a.Y + b.Y);
        end

        function r = minus(a, b)
            %MINUS Component-wise difference.
            r = a_point_value(a.X - b.X, a.Y - b.Y);
        end

        function r = times(a, k)
            %TIMES Scale a point by a scalar.
            r = a_point_value(a.X * k, a.Y * k);
        end

        function tf = eq(a, b)
            %EQ True if the points match within Tolerance.
            tf = abs(a.X - b.X) < a_point_value.Tolerance && ...
                 abs(a.Y - b.Y) < a_point_value.Tolerance;
        end

        function n = get.Norm(obj)
            n = hypot(obj.X, obj.Y);
        end

        function s = toString(obj)
            s = sprintf('(%g, %g)', obj.X, obj.Y);
        end
    end

    methods (Static)
        function p = fromPolar(r, theta)
            %FROMPOLAR Create a point from polar coordinates.
            p = a_point_value(r .* cos(theta), r .* sin(theta));
        end

        function p = origin()
            %ORIGIN Return the point (0, 0).
            p = a_point_value(0, 0);
        end

        function d = distance(a, b)
            %DISTANCE Euclidean distance between two points.
            d = hypot(a.X - b.X, a.Y - b.Y);
        end
    end
end
