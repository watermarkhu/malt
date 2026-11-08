function varargout = fv2(varargin)
    arguments (Input, Repeating)
        varargin
    end
    arguments (Output, Repeating)
        varargout
    end
end
function [r, varargout] = fvB(a, opts)
    arguments (Input)
        a (1,:) double {mustBePositive} = 1
        opts.Mode (1,1) string = "a"
        opts.Other = opts.Mode
    end
    arguments (Output)
        r
    end
    arguments (Output, Repeating)
        varargout
    end
    r = 1;
end
function r = fvC(a, opts)
    arguments
        a
        opts.alpha = 1
        opts.alpha2 = 2
        opts.?double
    end
    r = nested(a);
    function q = nested(z)
        arguments
            z
        end
        q = z;
    end
end
function r = fvD(opts)
    arguments
        opts.A (1,1) double = 1
        opts.B (1,1) double = opts.A
    end
    r = 1;
end
function r = fvE(opts, a)
    arguments
        opts.X = 1
        a = mynest(1)
    end
    r = 1;
    function q = mynest(z), q = z; end
end
function r = fvF(a)
    arguments
        a (1,1) double = 1
    end
    arguments (Input)
        a
    end
    r = a;
end
function r = fvG(a, opts)
    arguments
        a
        opts.?matlab.graphics.chart.primitive.Line = 1
    end
    r = 1;
end
function r = fvH(a, b)
    arguments (Input, Repeating)
        a
        b
    end
    arguments (Output)
        r
    end
    arguments (Input)
        a2
    end
end
