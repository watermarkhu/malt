function f6(x, y)
    arguments
        x (1,1) {mustBeNumeric}
        y {mustBeFinite(x), mustBeGreaterThan(y, x)}
    end
end
function [o] = f7(x)
    arguments (Output)
        o {mustBeGreaterThan(o, x)}
    end
end
function [o] = f8(x)
    arguments (Output)
        o {mustBeNumeric(1)}
    end
end
function f9(x,y)
    arguments
       x (2,:) 
       y (:,:,3)
    end
end
function f10(x)
    arguments
        x (1,1) = 1
    end
end
function f11(x)
    arguments
        x (1,:)
    end
end
function f12(x)
    arguments
        x (-1,2)
        x (1e2, 3)
        x (0x10, 2)
    end
end
