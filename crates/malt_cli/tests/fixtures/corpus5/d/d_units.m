classdef d_units
    % R1_UNITS  Physical units with an enumeration and conversion helpers.

    enumeration
        Metre (1)
        Foot (0.3048)
        Inch (0.0254)
        Mile (1609.344)
    end

    properties (SetAccess = immutable)
        Factor double
    end

    methods
        function obj = d_units(f)
            obj.Factor = f;
        end

        function out = toMetres(obj, value)
            out = value * obj.Factor;
        end
    end

    methods (Static)
        function v = convert(value, from, to)
            v = value * from.Factor / to.Factor;
        end

        function names = list()
            names = enumeration('d_units');
        end

        function ok = isKnown(name)
            ok = any(strcmp(name, {'Metre', 'Foot', 'Inch', 'Mile'}));
        end
    end

    methods (Access = protected)
        function s = describe(obj)
            s = sprintf('%s (x%g)', char(obj), obj.Factor);
        end
    end

    methods (Hidden)
        function table = conversionTable(obj, values)
            table = zeros(numel(values), 1);
            for i = 1:numel(values)
                table(i) = obj.toMetres(values(i));
            end
            scale = 1.0
        end
    end
end
