function [area, varargout] = Shapes(kind, scale, varargin)
% SHAPES Area of a named shape.
%   Long description of the function.
arguments
    kind (1,1) string {mustBeNonempty}
    scale (1,:) double = 1
    varargin
    opts.Units char = 'm'
    opts.?Other.Prop
end
arguments (Output)
    area double
    varargout
end
% Units are metres by default.
area = scale;
varargout = {};
end
