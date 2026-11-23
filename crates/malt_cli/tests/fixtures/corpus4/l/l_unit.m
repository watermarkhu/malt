classdef l_unit
    %L_UNIT Length units with conversion factors to metres.
    %   Members carry their printed symbol and the factor that turns a
    %   value in this unit into metres.
    %
    %   Example:
    %       u = l_unit.fromString("ft");
    %       m = u.convert(10, l_unit.metre);

    enumeration
        millimetre ('mm', 0.001)
        centimetre ('cm', 0.01)
        metre      ('m', 1)
        foot       ('ft', 0.3048)
        inch       ('in', 0.0254)
    end

    properties (SetAccess = private)
        Symbol (1,1) string
        ToMetres (1,1) double
    end

    methods
        function obj = l_unit(sym, factor)
            obj.Symbol = string(sym);
            obj.ToMetres = factor;
        end

        function v = convert(obj, value, target)
            %CONVERT Express value (given in obj) in the target unit.
            arguments
                obj
                value double
                target (1,1) l_unit
            end
            v = value * obj.ToMetres / target.ToMetres;
        end

        function s = toString(obj)
            s = obj.Symbol;
        end

        function tf = isMetric(obj)
            tf = any(obj == [l_unit.millimetre, l_unit.centimetre, l_unit.metre]);
        end
    end

    methods (Static)
        function u = fromString(str)
            %FROMSTRING Look up a unit by its symbol, case-insensitively.
            arguments
                str (1,:) char
            end
            names = enumeration('l_unit');
            idx = find(lower([names.Symbol]) == lower(string(str)), 1);
            if isempty(idx)
                error('l_unit:unknown', 'Unknown unit "%s".', str);
            end
            u = names(idx);
        end

        function names = allSymbols()
            %ALLSYMBOLS Symbols of every member, in declaration order.
            members = enumeration('l_unit');
            names = [members.Symbol];
        end
    end
end
