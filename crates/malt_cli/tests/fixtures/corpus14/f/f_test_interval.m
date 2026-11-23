classdef f_test_interval < matlab.unittest.TestCase
    properties
        Base
    end
    methods (TestMethodSetup)
        function makeBase(testCase)
            testCase.Base = b_interval_class(0, 10);
        end
    end
    methods (Test)
        function testWidth(testCase)
            testCase.verifyEqual(testCase.Base.width(), 10);
        end
        function testPlus(testCase)
            r = testCase.Base + 5;
            testCase.verifyEqual(r.Lo, 5);
            testCase.verifyEqual(r.Hi, 15);
        end
        function testScale(testCase)
            r = testCase.Base * -2;
            testCase.assertTrue(r.Lo < r.Hi);
            expected = -20;
            testCase.verifyEqual(r.Lo, expected);
        end
        function testContains(testCase)
            x = [-1 0 5 10 11];
            got = testCase.Base.contains(x);
            testCase.verifyEqual(got, logical([0 1 1 1 0]));
        end
    end
    methods (Test, TestTags = {'slow'})
        function testMany(testCase)
            for k = 1:100
                a = b_interval_class(k, k + 1);
                testCase.verifyTrue(a.contains(k + 0.5));
            end
        end
    end
end
