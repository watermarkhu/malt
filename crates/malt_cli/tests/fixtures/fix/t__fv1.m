function r = fv1(a, b, opts)
    arguments
        a (1,1) double = 1
        b (1,1) double
        opts.Name (1,1) string = "x"
        opts.Name (1,1) string
    end
    r = a;
end
function r = fv2(a, a2, varargin)
    arguments
        a
        a
        a2 {mustBeNumeric} = 2
    end
    arguments (Repeating)
        varargin
    end
    arguments (Repeating)
        x
    end
    r = 1;
end
function r = fv3(a, ~, opts)
    arguments
        a
        ~ double
        opts.X
        opts.Y = 3
        b
    end
    arguments (Input, Output)
        r
    end
end
function r = fv4(a, opts)
    arguments (Output)
        r
    end
    arguments (Input)
        a
        opts.?matlab.graphics.chart.primitive.Line
        opts.?double
    end
end
function r = fv5(x, opts)
    arguments (Input, Repeating)
        x (1,1) = 3
        opts.Foo
    end
    r = 1;
end
function r = fv6(x, name)
    arguments
        x {mustBeNumeric, eval('1')}
        name = nargin
    end
    function q = nested(), q = 1; end
    r = 1;
end
function r = fv7(a, b)
    arguments (Input = true)
        a
    end
    arguments (Output, Foo)
        r
    end
    r = 1;
end
