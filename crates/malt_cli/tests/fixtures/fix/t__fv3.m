function r = fvA(opts)
    arguments
        opts.string (1,1) string = "a"
        opts.double {mustBeNumeric} = 1
    end
    r = 1;
end
function r = fvB(n, opts)
    arguments (Input)
        n
        opts.?double = 3
    end
    r = 1;
end
function r = fvC(n, opts)
    arguments (Input)
        n {mustBeNumeric}
        opts.?double {mustBePositive}
    end
    r = 1;
end
function r = fvD(n, opts)
    arguments (Input)
        n
        opts.?matlab.graphics.chart.primitive.Line
    end
    r = nargin;
end
function r = fvE(n, m)
    arguments
        n (1,1) {mustBeNumeric} = nargin
        m (1,1) = inputname(1)
    end
    r = 1;
end
function r = fvF(n)
    arguments
        n (1,1) {mustBeNumeric} = narginchk(1,2)
    end
    r = 1;
end
function r = fvG(n)
    arguments (Input)
        n
        k
    end
    arguments (Input, Foo)
        n
    end
    r = 1;
end
