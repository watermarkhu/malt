classdef Foo < Bar
    methods
        function obj = Foo()
            x = obj@Bar();
        end
    end
end
