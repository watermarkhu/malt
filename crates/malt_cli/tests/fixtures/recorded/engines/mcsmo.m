classdef Foo < Bar
    methods
        function [obj, x] = Foo()
            [obj, x] = obj@Bar(1);
        end
    end
end
