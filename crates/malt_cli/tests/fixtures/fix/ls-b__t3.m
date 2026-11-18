function varargout = t3(a, ~, c)
    arguments
        a
        ~
        c (1,1) {mustBeNumeric(c)}
    end
    arguments (Output, Repeating)
        varargout
    end
    varargout{1}=1;
end
