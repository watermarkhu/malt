function t2(a, b, varargin)
    arguments
        a (1,1) {mustBePositive(b)}
        b (1,1) {mustBePositive(a)}
    end
    arguments (Repeating)
        varargin
        c
    end
end
