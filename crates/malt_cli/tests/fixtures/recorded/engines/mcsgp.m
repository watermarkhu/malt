classdef Foo
    methods
        function validateW(v)
            s = struct('a', v.b);
        end
    end
    methods (Abstract)
        set.k(obj, v)
        v = get.k2(obj)
    end
end
