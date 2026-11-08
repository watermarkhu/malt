function v1(x, y, z)
    arguments
        x (1,1) {mustBeLessThan(x, pi)}
        y (1,1) {mustBeLessThan(y, Inf), mustBeGreaterThan(y, x)}
        z (1,1) {mustBeInRange(z, 0, numel(y)), mustBeMember(z, [1 2 3])}
    end
end
function v2(x, y)
    arguments
        x {mustBeFinite(y), mustBeFoo(y)}
        y {mustBeA(y, 'double'), mustBeMember(y, MyEnum.A)}
    end
end
function v3(x)
    arguments
        x {mustBeA(x, ?matlab.System)}
    end
end
function v4(x, y)
    arguments
        x {@(v) v > 0}
        y {mustBeGreaterThan(y, x.a)}
    end
end
function v5(a, opts)
    arguments
        a
        opts.N (1,1) {mustBeLessThan(opts.N, a)}
    end
end
function v6(x)
    arguments
        x {mustBeNumeric(x), mustBeReal(x), mustBeFinite(x)}
    end
end
