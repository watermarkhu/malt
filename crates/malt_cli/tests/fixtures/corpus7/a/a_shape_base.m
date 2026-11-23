classdef a_shape_base < handle & matlab.mixin.Heterogeneous
    % R3_SHAPE_BASE  Abstract base for drawable shapes with a listener hook.

    properties (SetAccess = protected)
        Name string = "shape"
        Visible logical = true
    end

    properties (Access = protected)
        Listeners event.listener = event.listener.empty
    end

    events
        Moved
    end

    methods (Abstract)
        a = area(obj)
        p = perimeter(obj)
    end

    methods
        function obj = a_shape_base(name)
            if nargin > 0
                obj.Name = name;
            end
        end

        function show(obj)
            if ~obj.Visible
                return
            end
            fprintf('%s: area %.3f, perimeter %.3f\n', obj.Name, obj.area(), obj.perimeter());
        end

        function hide(obj)
            obj.Visible = false;
        end
    end

    methods (Sealed)
        function s = summary(obj)
            s = struct('name', char(obj.Name), 'area', obj.area, 'perimeter', obj.perimeter);
        end
    end

    methods (Access = protected)
        function onMove(obj, dx)
            notify(obj, 'Moved', eventdata_(dx));
        end
    end
end

function ed = eventdata_(dx)
    ed = dx;
end
