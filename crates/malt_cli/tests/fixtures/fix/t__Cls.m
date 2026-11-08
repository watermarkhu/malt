classdef Cls
    methods
        function r = m(obj, x, opts)
            arguments
                obj
                x (1,1) double
                opts.Foo = 1
            end
            r = x;
        end
        function r = n(obj, varargin)
            arguments (Repeating)
                varargin
            end
            r = 1;
        end
        function r = Cls(~, x)
            arguments
                ~
                x
            end
        end
    end
end
