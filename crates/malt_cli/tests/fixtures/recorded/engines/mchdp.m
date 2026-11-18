classdef Foo < handle
    properties
        p1 = handle()
        p2 = onCleanup(@cleanup)
        p3 = containers.Map()
        p4 = timer()
        p5 = Foo()
    end
end
