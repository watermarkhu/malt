classdef Foo < Bar
    methods
        function obj = Foo()
            obj = obj@NotSuper();
        end
    end
end
