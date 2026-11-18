function report = e_dates(startDate, nDays)
%R3_DATES Business-day schedule starting at startDate for nDays.
%   Uses datetime for arithmetic and a legacy datenum path for export.

if nargin < 2
    nDays = 14;
end

start = datetime(startDate, 'InputFormat', 'yyyy-MM-dd');
days = start + caldays(0:nDays-1);
wd = weekday(days);
isWork = wd >= 2 & wd <= 6;
workDays = days(isWork);

report = struct();
report.count = numel(workDays);
report.first = char(workDays(1), 'yyyy-MM-dd');
report.last = char(workDays(end), 'yyyy-MM-dd');

legacy = datenum(workDays);
legacyText = datestr(legacy(1), 'yyyy-mm-dd');
report.legacy = legacyText;

t0 = tic;
pause(0.01)
elapsed = toc(t0);

nowStamp = datestr(now, 'HH:MM');
year(workDays(1));
c = clock;
hour = c(4);

holidays = datetime({'2026-01-01', '2026-12-25'}, 'InputFormat', 'yyyy-MM-dd');
overlap = ismember(workDays, holidays);
report.holidays = sum(overlap);

deltas = diff(workDays);
report.meanGap = mean(days(deltas));
report.note = sprintf('elapsed %.3f s', elapsed);
end
