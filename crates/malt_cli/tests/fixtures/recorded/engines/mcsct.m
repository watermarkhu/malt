classdef Foo < Bar
    methods
        function obj = Foo()
            obj = 1 + obj@Bar();
        end
    end
end
