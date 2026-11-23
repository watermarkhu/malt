classdef defnu_p52_app_header_comment
    properties (Access = public)
        UIFigure
        pushButton
    end

    methods (Access = private)
        % Callbacks that handle component events
        function appButtonPushed(app, event)
            disp(app.UIFigure);
        end
    end
end
