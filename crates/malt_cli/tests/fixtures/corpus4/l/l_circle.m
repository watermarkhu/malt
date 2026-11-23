classdef l_circle < l_shape
    %L_CIRCLE Circle defined by a radius and a centre point.
    %   The radius is validated on assignment through its property
    %   attributes. Area is computed on demand, so it is never stale.

    properties
        Radius (1,1) double {mustBePositive} = 1
        Centre (1,2) double = [0 0]
    end

    properties (Dependent)
        Area
        Diameter
    end

    properties (Hidden, Transient)
        LastTest double = NaN
    end

    methods
        function obj = l_circle(radius, centre, varargin)
            %L_CIRCLE Construct a circle, optionally with a centre.
            obj@l_shape("circle");
            if nargin >= 1
                obj.Radius = radius;
            end
            if nargin >= 2
                obj.Centre = centre;
            end
            if numel(varargin) >= 2 && strcmp(varargin{1}, 'Name')
                obj.Name = varargin{2};
            end
        end

        function a = get.Area(obj)
            a = pi * obj.Radius ^ 2;
        end

        function d = get.Diameter(obj)
            d = 2 * obj.Radius;
        end

        function a = area(obj)
            a = obj.Area;
        end

        function p = perimeter(obj)
            p = 2 * pi * obj.Radius;
        end

        function scale(obj, factor)
            %SCALE Multiply the radius by factor and announce the change.
            obj.Radius = obj.Radius * factor;
            obj.notifyChanged();
        end

        function tf = contains(obj, xy)
            %CONTAINS Logical mask of points inside the circle.
            arguments
                obj
                xy (:,2) double
            end
            d = vecnorm(xy - obj.Centre, 2, 2)';
            obj.LastTest = numel(d);
            tf = l_helpers2(d, obj.Radius);
        end

        function s = toStruct(obj)
            %TOSTRUCT Snapshot of the public geometry.
            s = struct('Radius', obj.Radius, ...
                'Centre', obj.Centre, ...
                'Area', obj.Area);
        end
    end

    methods (Access = private)
        function s = legacyArea(obj)
            %LEGACYAREA Area computed the old way; kept for reference.
            s = pi * obj.Radius * obj.Radius;
        end
    end
end
