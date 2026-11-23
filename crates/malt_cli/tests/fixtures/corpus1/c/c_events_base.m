classdef (Abstract) c_events_base < handle
    %C_EVENTS_BASE Abstract base for workers that report progress.
    %   Subclasses must implement doStep and may override describe.

    properties (SetAccess = protected)
        Label (1,:) char
    end

    methods
        function obj = c_events_base(label)
            if nargin > 0
                obj.Label = label;
            end
        end

        function text = describe(obj)
            % Default one-line description of the worker
            text = sprintf('[%s]', obj.Label);
        end
    end

    methods (Abstract)
        doStep(obj, k)
    end

    methods (Abstract, Access = protected)
        result = finalise(obj)
    end
end
