classdef g_unit_tests < matlab.unittest.TestCase
    % R1_UNIT_TESTS  Tests for the rounding helper in the probe corpus.

    properties
        Tolerance = 1e-9
        Sample
    end

    methods (TestMethodSetup)
        function makeSample(testCase)
            testCase.Sample = [1.2345, 2.5, -3.75];
        end
    end

    methods (Test)
        function roundsDown(testCase)
            actual = round(testCase.Sample(1), 2);
            testCase.verifyEqual(actual, 1.23, 'AbsTol', testCase.Tolerance);
        end

        function keepsSign(testCase)
            actual = fix(testCase.Sample(3));
            testCase.verifyEqual(actual, -3)
            testCase.assertTrue(actual < 0);
        end

        function emptyInput(testCase)
            x = [];
            testCase.verifyEmpty(x);
            unused = 5;
        end
    end

    methods (Test, TestTags = {'slow'})
        function bigVector(testCase)
            v = 1:1e6;
            testCase.verifySize(v, [1 1e6]);
        end
    end
end
