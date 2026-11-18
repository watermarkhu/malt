function f4(x, opts)
    arguments
        x
        opts.A (1,1) {mustBeNumeric(A)}
        opts.B (1,1) {mustBeNumeric(opts.B), mustBeLessThan(opts.A, B)}
    end
end
