classdef b_atunk_superclass < b_unknown_base
    % Attribute names a superclass may define: silent with any other superclass.

    properties (Mystery)
        Count
    end

    methods (Bogus)
        function r = value(~)
            r = 1;
        end
    end
end
