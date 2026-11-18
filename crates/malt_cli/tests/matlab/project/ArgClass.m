classdef ArgClass
    % ARGCLASS A class whose methods have arguments blocks.
    methods
        function [out, n] = withArgs(obj, x, y)
            arguments
                obj
                x (1,1) double {mustBePositive}
                y (1,:) double = [1 2]
            end
            out = x;
            n = numel(y);
        end
        function out = named(obj, opts)
            arguments
                obj
                opts.Name (1,1) string = "d"
                opts.Count (1,1) double = 2
            end
            out = opts.Name;
        end
    end
end
