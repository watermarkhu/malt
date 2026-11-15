classdef Circle < geo.Point
    % A circle: a point and a radius.
    properties
        Radius (1,1) double {mustBeNonnegative} = 1
    end
end
