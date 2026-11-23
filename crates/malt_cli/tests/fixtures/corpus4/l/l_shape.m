classdef l_shape < handle
    %L_SHAPE Abstract base class for two-dimensional shapes.
    %   Subclasses supply the geometry through the abstract methods.
    %   DESCRIBE is a template method that assembles the report string
    %   from the subclass results, so subclasses rarely override it.
    %
    %   Example:
    %       c = l_circle(2);
    %       disp(c.describe())

    properties (SetAccess = protected)
        Name (1,1) string = "shape"
    end

    properties (Constant)
        Version (1,1) double = 1
    end

    properties (Access = protected)
        Cache struct = struct()
    end

    events
        Changed
    end

    methods (Abstract)
        a = area(obj)
        p = perimeter(obj)
    end

    methods
        function obj = l_shape(name)
            %L_SHAPE Construct a shape with an optional name.
            if nargin > 0
                obj.Name = name;
            end
        end

        function s = describe(obj)
            %DESCRIBE Template method producing a one-line summary.
            s = sprintf('%s: area=%.3f perimeter=%.3f', ...
                obj.Name, obj.area(), obj.perimeter());
            obj.Cache.last = s;
        end

        function tf = isLarger(obj, other)
            %ISLARGER True when this shape has the larger area.
            arguments
                obj
                other (1,1) l_shape
            end
            tf = obj.area() > other.area();
        end

        function notifyChanged(obj)
            %NOTIFYCHANGED Broadcast the Changed event to listeners.
            notify(obj, 'Changed');
        end
    end

    methods (Sealed)
        function n = nameOf(obj)
            %NAMEOF Return the shape name.
            n = obj.Name;
        end
    end

    methods (Static)
        function v = apiVersion()
            %APIVERSION Version of the shape interface.
            v = l_shape.Version;
        end
    end

    methods (Access = protected)
        function checkPositive(~, x, label)
            %CHECKPOSITIVE Raise an error unless x is a positive scalar.
            if ~(isnumeric(x) && isscalar(x) && x > 0)
                error('l_shape:badValue', '%s must be a positive scalar.', label);
            end
        end
    end
end
