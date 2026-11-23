classdef e_tests_stats < matlab.unittest.TestCase
    %E_TESTS_STATS Unit tests for the statistics helpers in this project.
    %   Run with: results = runtests('e_tests_stats')

    properties
        TestData
        Tolerance = 1e-10
    end

    methods (TestClassSetup)
        function addProjectPath(testCase) %#ok<MANU>
            here = fileparts(mfilename('fullpath'));
            addpath(fullfile(here, '..', 'src'));
        end
    end

    methods (TestMethodSetup)
        function makeData(testCase)
            rng(7);
            testCase.TestData = randn(200, 1);
        end
    end

    methods (TestMethodTeardown)
        function clearData(testCase)
            testCase.TestData = [];
        end
    end

    methods (Test)
        function meanMatchesBuiltin(testCase)
            expected = mean(testCase.TestData);
            actual = mean(testCase.TestData, 'omitnan');
            testCase.verifyEqual(actual, expected, 'AbsTol', testCase.Tolerance);
        end

        function standardErrorIsSmall(testCase)
            se = std(testCase.TestData) / sqrt(numel(testCase.TestData));
            testCase.verifyLessThan(se, 0.1);
        end

        function emptyInputGivesNaN(testCase)
            out = mean([], 'omitnan');
            testCase.verifyTrue(isnan(out));
        end

        function vectorOfTwoIsCorrect(testCase)
            x = [1 2 3 4];
            testCase.verifyEqual(median(x), 2.5);
        end

        function rejectsNonNumeric(testCase)
            testCase.verifyError(@() mean({'a', 'b'}), 'MATLAB:UnableToConvert');
        end

        function fastPathOnlyOnLinux(testCase)
            testCase.assumeTrue(isunix, 'Skipped: needs a Unix-like system.');
            [~, hostname] = system('hostname');
            testCase.verifyNotEmpty(strtrim(hostname));
        end

        function histogramCountsAddUp(testCase)
            [counts, ~] = histcounts(testCase.TestData, 10);
            testCase.verifyEqual(sum(counts), numel(testCase.TestData));
        end

        function quantilesAreOrdered(testCase)
            q = quantile(testCase.TestData, [0.25 0.5 0.75]);
            testCase.verifyTrue(issorted(q));
        end
    end

    methods (Test, ParameterCombination = 'sequential')
        function scaleInvariance(testCase, scale)
            x = testCase.TestData;
            r1 = std(x);
            r2 = std(scale * x);
            testCase.verifyEqual(r2, abs(scale) * r1, 'RelTol', 1e-9);
        end
    end

    properties (TestParameter)
        scale = struct('small', 0.01, 'unit', 1, 'large', 1000);
    end
end
