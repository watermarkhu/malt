classdef f_class_hierarchy < f_base_shape & matlab.mixin.Copyable
%F_CLASS_HIERARCHY Rectangle with protected sides, built on f_base_shape.
%   Example:
%     r = f_class_hierarchy(3, 4);
%     r2 = copy(r);
%     resize(r2, 10, 2)
%     area(r2)

    properties (SetAccess = protected)
        Width (1,1) double {mustBePositive} = 1
        Height (1,1) double {mustBePositive} = 1
    end

    methods
        function obj = f_class_hierarchy(w, h)
            % Constructor accepts optional width and height
            obj@f_base_shape();
            obj.Label = 'rectangle';
            if nargin >= 1
                obj.Width = w;
            end
            if nargin >= 2
                obj.Height = h;
            end
        end

        function a = area(obj)
            a = obj.Width * obj.Height;
        end

        function resize(obj, w, h)
            % Sides are protected, so only the class itself can change them
            obj.Width = w;
            obj.Height = h;
        end

        function new = copy(obj)
            % Use the Copyable implementation, then keep the same label
            new = copy@matlab.mixin.Copyable(obj);
        end

        function tf = isequal(obj, other)
            % Two rectangles are equal when their dimensions match
            tf = isa(other, 'f_class_hierarchy') && ...
                obj.Width == other.Width && obj.Height == other.Height;
        end
    end
end
