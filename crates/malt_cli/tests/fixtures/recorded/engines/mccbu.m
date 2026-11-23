classdef Foo < Bar
    properties
        y
    end
    methods
        function obj = Foo()
            obj.y = 1;
            obj = obj@Bar();
        end
    end
end
