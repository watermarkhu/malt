classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@Bar();
            obj = obj@Bar();
        end
    end
end
