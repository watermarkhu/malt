classdef test1 < matlab.unittest.TestCase
    methods (Test)
        function testAdd(testCase)
            actual = 1 + 1;
            testCase.verifyEqual(actual, 2);
        end
        function testEmpty(testCase)
            testCase.assertEmpty([]);
        end
    end
    methods (TestMethodSetup)
        function setup(testCase)
            testCase.addTeardown(@() disp('done'));
        end
    end
end
