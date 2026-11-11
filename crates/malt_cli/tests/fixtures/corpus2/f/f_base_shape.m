classdef (Abstract) f_base_shape < handle
%F_BASE_SHAPE Abstract base for planar shapes.
%   Subclasses must implement AREA. DESCRIBE is shared by all shapes.

    properties
        Label = 'shape'
    end

    methods (Abstract)
        a = area(obj)
    end

    methods
        function describe(obj)
            % Print a one-line summary of the shape
            fprintf('%s (%s) with area %.3f\n', class(obj), obj.Label, area(obj));
        end
    end
end
