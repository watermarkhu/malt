function tests = a_tests_calc
%A_TESTS_CALC Unit tests for the calculation helpers.
%   Run with:  results = runtests('a_tests_calc')

tests = functiontests(localfunctions);

end

function setupOnce(testCase)
% Shared settings for every test in this file
testCase.TestData.tolerance = 1e-10;
end

function setup(testCase)
% Fresh scratch folder for each test
testCase.TestData.workdir = tempname;
mkdir(testCase.TestData.workdir);
end

function teardown(testCase)
if isfolder(testCase.TestData.workdir)
    rmdir(testCase.TestData.workdir, 's');
end
end

function testMeanOfRange(testCase)
actual = mean(1:10);
expected = 5.5;
verifyEqual(testCase, actual, expected)
end

function testExpLogRoundTrip(testCase)
x = linspace(0.1, 2*pi, 50);
verifyEqual(testCase, exp(log(x)), x, 'RelTol', 1e-12)
end

function testWriteAndReadFile(testCase)
fname = fullfile(testCase.TestData.workdir, 'out.txt');
fid = fopen(fname, 'w');
fprintf(fid, 'hello\n');
fclose(fid);
txt = fileread(fname);
verifyEqual(testCase, txt, sprintf('hello\n'))
end

function testStdNearZero(testCase)
actual = std([2 2 2 2]);
verifyEqual(testCase, actual, 0, 'AbsTol', testCase.TestData.tolerance)
end

function testBadOptionThrows(testCase)
verifyError(testCase, @() validatestring('zzz', {'alpha', 'beta'}), ...
    'MATLAB:validatestring:UnrecognizedString')
end
