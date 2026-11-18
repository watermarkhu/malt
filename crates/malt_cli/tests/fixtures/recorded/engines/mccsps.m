classdef Foo
    properties (Constant)
        K = 1
    end
    methods
        function m(obj)
            Foo.K = 5;
        end
    end
end
