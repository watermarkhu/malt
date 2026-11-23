classdef Foo < Bar
    methods
        function obj = Foo()
            q@Bar();
        end
    end
end
