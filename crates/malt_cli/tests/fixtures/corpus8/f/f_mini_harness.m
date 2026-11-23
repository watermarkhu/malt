% R4_MINI_HARNESS  A tiny test harness written as a script.
% Each case is a function handle; failures are collected, not thrown.

cases = {
    'adds', @() assertEqual_(1 + 1, 2)
    'divides', @() assertEqual_(6 / 3, 2)
    'fails on purpose', @() assertEqual_(1, 2)
    'throws', @() error('r4:harness:boom', 'boom happened')
};

passed = 0;
failed = {};
for c = 1:size(cases, 1)
    name = cases{c, 1};
    body = cases{c, 2};
    try
        body();
        passed = passed + 1;
    catch err
        failed{end+1} = sprintf('%s (%s)', name, err.identifier);
    end
end

fprintf('%d passed, %d failed\n', passed, numel(failed))
for f = 1:numel(failed)
    fprintf('  FAILED %s\n', failed{f})
end

exist f_mini_harness
exist 'a_kernel.m'
x = exist('b_vending');

try
    assertEqual_(3, 4)
catch err2
    rethrow(err2)
end

function assertEqual_(a, b)
    if a ~= b
        throw(MException('r4:harness:mismatch', 'expected %g got %g', b, a));
    end
end
