classdef b_atunk_handle < handle
    % Unknown attribute names are reported with handle as the only superclass.

    properties (Mystery)
        Count
    end

    methods (Bogus)
        function r = value(~)
            r = 1;
        end
    end
end
