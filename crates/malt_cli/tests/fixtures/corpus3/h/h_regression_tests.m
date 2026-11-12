%% H_REGRESSION_TESTS Regression harness for the signal toolbox wrappers
% Run from the project root. Each row of CASES: name, function handle, args, expected.
%
% TODO: replace with matlab.unittest once the CI image has it
clear; close all;  % old habit, keep

%% cases
cases = {
    'sum basic',      @sum,        {[1 2 3]},                     6
    'mean matrix',    @mean,       {magic(4)},                    8.5
    'strlen',         @strlength,  {"abc"},                       3
    'interp',         @(x) interp1(1:3, [10 20 30], x), {2.5},    25
    'max vec',        @max,        {[4 9 2]},                     9
};

tol = 1e-9;
failures = {};
nPass = 0;
t0 = datetime('now');

%% run
for i = 1:size(cases, 1)
    name = cases{i, 1};
    fn = cases{i, 2};
    args = cases{i, 3};
    want = cases{i, 4};
    try
        got = fn(args{:});
        if isnumeric(want) && isnumeric(got) && all(size(got) == size(want))
            ok = max(abs(got(:) - want(:))) <= tol;
        else
            ok = isequal(got, want);
        end
        if ok
            nPass = nPass + 1;
        else
            failures{end+1} = sprintf('%s: got %s', name, mat2str(got)); %#ok<SAGROW>
        end
    catch
        failures{end+1} = sprintf('%s: threw "%s"', name, lasterr); %#ok<SAGROW>
    end
end

%% extra checks: evalc, exist, assert
out = evalc('disp(pi)');
if ~contains(out, '3.1416')
    failures{end+1} = 'evalc disp mismatch';
end

if exist('h_signal_class', 'file') ~= 2
    failures{end+1} = 'h_signal_class not on path';
end

try
    assert(numel(cases) > 0, 'no cases defined');
catch err
    fprintf('setup: %s\n', err.message);
end

%% summary
nTotal = size(cases, 1) + 2;
fprintf('\n=== regression summary ===\n');
fprintf('passed %d / %d\n', nPass, nTotal);
fprintf('failed %d\n', numel(failures));
for k = 1:numel(failures)
    fprintf('  - %s\n', failures{k});
end
fprintf('elapsed %s\n', char(datetime('now') - t0));
status = report_local(failures);
fprintf('status: %s\n', status);

function s = report_local(failures)
%REPORT_LOCAL one-line status string
if isempty(failures)
    s = 'OK';
else
    s = sprintf('FAIL (%d)', numel(failures));
end
end
