function table_out = schedule_map(shifts, startDate)
%SCHEDULE_MAP Builds a weekly shift table per employee from a shift list.
%   shifts is a struct array with fields name, day (1 to 7) and hours.

if nargin < 2
    startDate = datetime(2026, 1, 5);   % a Monday
end

byName = containers.Map();
for k = 1:numel(shifts)
    who = shifts(k).name;
    if ~isKey(byName, who)
        byName(who) = zeros(1, 7);
    end
    days = byName(who);
    days(shifts(k).day) = days(shifts(k).day) + shifts(k).hours;
    byName(who) = days;
end

names = sort(keys(byName));
rows = numel(names);
weekStart = repmat(startDate, rows, 1);
totals = zeros(rows, 1);
for r = 1:rows
    hours = byName(names{r});
    totals(r) = sum(hours);
    if totals(r) > 40
        warning('schedule:overtime', '%s works %g hours', names{r}, totals(r));
    end
end

shiftEnd = weekStart + days(6);
span = between(weekStart(1), shiftEnd(1));
fprintf('week of %s to %s\n', char(weekStart(1)), char(shiftEnd(1)));

table_out = table(string(names), totals, 'VariableNames', {'Name', 'Hours'});
table_out.Week = repmat(startDate, rows, 1);
weekend = isweekend(startDate + caldays(0:6));
nWeekend = nnz(weekend);

labels = cell(1, 7);
for d = 1:7
    labels{d} = datestr(startDate + d - 1, 'ddd');
end
disp(strjoin(labels, ' | '))
x = 1;
y = x;
clear x
end
