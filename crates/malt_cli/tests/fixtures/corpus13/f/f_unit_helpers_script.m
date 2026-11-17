%% Helper script: prints a sorted table of test names and durations
tests = struct('name', {'parse', 'render', 'export', 'import'}, ...
    'seconds', {0.4, 1.2, 0.05, 2.7});

[~, idx] = sort([tests.seconds], 'descend');
ranked = tests(idx);
for i = 1:numel(ranked)
    row = sprintf('%-10s %6.2f', ranked(i).name, ranked(i).seconds);
    disp(row)
end

slow = ranked([ranked.seconds] > 1);
if isempty(slow)
    msg = 'no slow tests';
else
    msg = sprintf('%d slow tests', numel(slow));
end
disp(msg)
total = sum([tests.seconds]);
fprintf('total %.2f s\n', total)
avg = total / numel(tests);
result = struct('total', total, 'avg', avg);
result.slowest = ranked(1).name;
