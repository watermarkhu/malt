classdef d_app1 < matlab.apps.AppBase
    methods (Access = private)
        function r = helper(app, x)
            r = x;
        end
        function r = m(app)
            f = @() helper + 1;
            r = f();
        end
    end
end
