classdef (Sealed) Widget < handle & matlab.mixin.Copyable
% WIDGET A widget with a size and a name.
%{
Block text.
  Indented.
%}

properties (Access = private, Constant)
    % count of widgets
    Count (1,1) double = 3
    Name char
end

properties (Dependent)
    Size
end

methods (Static)
    function w = make(x)
        w = Widget(x);
    end
end

methods
    function obj = Widget(v)
        obj.Name = v;
    end
    function r = get.Size(obj)
        r = 1;
    end
    function set.Size(obj, v)
    end
    function show(obj, x, y)
        % shows the widget
    end
end

enumeration
    Red (1)
    % the green one
    Green (2)
end

events
    Changed
end
end
