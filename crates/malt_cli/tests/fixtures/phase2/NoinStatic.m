classdef NoinStatic
    methods (Static = TRUE)
        function r = f(x)
            r = x;
        end
    end
end
