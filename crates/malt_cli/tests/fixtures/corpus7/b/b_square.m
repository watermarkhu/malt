classdef b_square < a_shape_base
    % R3_SQUARE  Square shape, a concrete subclass of a_shape_base.

    properties
        Side (1,1) double {mustBePositive} = 1
    end

    properties (Dependent)
        Diagonal
    end

    methods
        function obj = b_square(side, name)
            arguments
                side (1,1) double {mustBePositive}
                name (1,1) string = "square"
            end
            obj@a_shape_base(name);
            obj.Side = side;
        end

        function a = area(obj)
            a = obj.Side ^ 2;
        end

        function p = perimeter(obj)
            p = 4 * obj.Side;
        end

        function d = get.Diagonal(obj)
            d = obj.Side * sqrt(2);
        end

        function grow(obj, amount)
            obj.Side = obj.Side + amount;
            obj.onMove(amount);
        end
    end

    methods (Static)
        function sq = unit()
            sq = b_square(1, "unit");
        end
    end
end
