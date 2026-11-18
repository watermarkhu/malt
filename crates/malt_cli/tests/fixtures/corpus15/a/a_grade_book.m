function [avg, report] = a_grade_book(names, scores, varargin)
% a_grade_book - compute averages, old style
global GRADE_CUTOFF
if isempty(GRADE_CUTOFF)
    GRADE_CUTOFF = 60;
end
nstud = length(names);
report = {};
avg = [];
for i = 1:nstud
    s = scores(i, :);
    if length(s) == 0
        continue
    end
    m = sum(s) / length(s);
    avg(i) = m;
    if m >= GRADE_CUTOFF & m < 90
        grade = 'pass';
    elseif m >= 90
        grade = 'distinction';
    else
        grade = 'fail';
    end
    report{end+1} = sprintf('%s: %5.1f %s', names{i}, m, grade);
end
best = find(avg == max(avg));
if ~isempty(best)
    fprintf('Best: %s\n', names{best(1)});
end
n_fail = length(find(avg < GRADE_CUTOFF));
if n_fail > 0, disp(['Failures: ' num2str(n_fail)]), end
opts = varargin;
if nargin > 2 && strcmp(opts{1}, 'sort')
    [avg, idx] = sort(avg);
    report = report(idx);
end
end
