function [a,b] = o1(x)
    arguments (Output)
        a {mustBeLessThan(a, pi)}
        b {mustBeLessThan(b, a), mustBeLessThan(b, Inf)}
    end
end
function varargout = o2(x)
    arguments (Output, Repeating)
        varargout {mustBeNumeric(varargout)}
    end
end
function [a, varargout] = o3(x)
    arguments (Output)
        a
    end
    arguments (Output, Repeating)
        varargout
    end
end
function [a, b] = o4(x)
    arguments (Output)
        a
    end
    arguments (Output)
        b
    end
end
function f(x, varargin)
    arguments
        x
    end
    arguments (Repeating)
        varargin
    end
end
function g(x, y, varargin)
    arguments
        x
    end
    arguments (Repeating)
        y
    end
    arguments (Repeating)
        varargin
    end
    arguments
        opts.A
    end
    arguments
        ~
    end
end
