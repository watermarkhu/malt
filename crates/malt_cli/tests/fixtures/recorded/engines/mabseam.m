classdef Foo < handle
    methods (Abstract, Sealed)
        r = m(obj)
    end
end
