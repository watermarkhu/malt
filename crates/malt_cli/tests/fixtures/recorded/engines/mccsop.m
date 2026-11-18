classdef Foo
    properties (Constant)
        x = 5
    end
    methods
        function obj = Foo()
            obj.x = 6;
        end
    end
end
