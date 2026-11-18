classdef c_quantity
    % c_quantity Magnitude with a unit label; value semantics.

    properties (SetAccess = private)
        Magnitude (1,1) double
        Unit (1,:) char = 'one'
    end

    properties (Dependent)
        IsDimensionless
    end

    properties (Constant, Access = private)
        KnownUnits = {'one', 'm', 's', 'kg', 'm/s'}
    end

    methods
        function obj = c_quantity(mag, unit)
            if nargin == 0
                return
            end
            if ~ismember(unit, c_quantity.KnownUnits)
                error('c_quantity:unknownUnit', 'Unit ''%s'' is not known.', unit);
            end
            obj.Magnitude = mag;
            obj.Unit = unit;
        end

        function tf = get.IsDimensionless(obj)
            tf = strcmp(obj.Unit, 'one');
        end

        function c = plus(a, b)
            c = c_quantity.requireSameUnit(a, b);
            c.Magnitude = a.Magnitude + b.Magnitude;
        end

        function c = minus(a, b)
            c = c_quantity.requireSameUnit(a, b);
            c.Magnitude = a.Magnitude - b.Magnitude;
        end

        function c = times(a, b)
            if isnumeric(a)
                c = c_quantity(a * b.Magnitude, b.Unit);
            elseif isnumeric(b)
                c = c_quantity(a.Magnitude * b, a.Unit);
            else
                error('c_quantity:mixedProduct', 'Only scalar scaling is supported.');
            end
        end

        function s = toString(obj)
            s = sprintf('%g %s', obj.Magnitude, obj.Unit);
        end

        function disp(obj)
            disp(['c_quantity: ' toString(obj)])
        end
    end

    methods (Static, Access = private)
        function c = requireSameUnit(a, b)
            if ~strcmp(a.Unit, b.Unit)
                error('c_quantity:unitMismatch', 'Cannot combine %s and %s.', a.Unit, b.Unit);
            end
            c = c_quantity(0, a.Unit);
        end
    end
end
