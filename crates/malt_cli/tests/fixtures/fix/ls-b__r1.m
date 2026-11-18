function r1(x, y, varargin)
    arguments
        x
        y
    end
    arguments (Repeating)
        varargin
    end
end
function r2(x, varargin)
    arguments
        x
    end
    arguments (Repeating)
        varargin
    end
end
function r3(x, y, varargin)
    arguments
        x
    end
    arguments (Repeating)
        y
        varargin
    end
end
function r4(x, y, z)
    arguments (Repeating)
        y
    end
    arguments
        z
    end
end
