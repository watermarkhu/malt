%% Smoke tests for the unit helpers, written as a script
addpath(fullfile(fileparts(mfilename('fullpath')), 'helpers'));
cleanup = onCleanup(@() disp('tests done'));

results = struct('name', {}, 'passed', {}, 'message', {});
tests = {
    'sum of two', @() assert(1 + 1 == 2)
    'string concat', @() assert(strcmp(['a' 'b'], 'ab'))
    'bad unit', @() convertOrFail('zz')
    };

for t = 1:size(tests, 1)
    name = tests{t, 1};
    body = tests{t, 2};
    try
        body();
        ok = true;
        msg = '';
    catch ME
        ok = false;
        msg = ME.message;
        if strcmp(ME.identifier, 'convert:badUnit')
            msg = ['expected: ' msg];
        end
    end
    results(end+1) = struct('name', name, 'passed', ok, 'message', msg);
end

nFail = sum(~[results.passed]);
if nFail > 0
    fprintf('%d of %d failed\n', nFail, numel(results));
else
    fprintf('all %d passed\n', numel(results));
end

try
    error('suite:done', 'finished with %d failures', nFail);
catch err
    disp(err.identifier);
end

e = MException('suite:custom', 'custom %s', 'failure');
try
    throw(e)
catch err2
    err2 = err2;
end
function convertOrFail(u)
    error('convert:badUnit', 'bad unit %s', u);
end
