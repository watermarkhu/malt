function [a, b] = t1(x, y, opts)
    arguments
        x (1,1) double = 1
        y (1,1) double
        opts.Foo (1,1) double = 2
        opts.Bar (1,1) double {mustBeGreaterThan(opts.Bar, x)} = 3
    end
    arguments (Output)
        a (1,1) double = 5
        b (1,:) double {mustBeNumeric}
    end
    a = 1; b = 2;
end
