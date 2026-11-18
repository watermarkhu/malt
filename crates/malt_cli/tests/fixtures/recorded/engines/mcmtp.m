classdef Foo < matlab.unittest.TestCase
    methods (TestParameterDefinition)
        function p = pdef(~)
            p = {1, 2};
        end
    end
end
