function report = e_modern_strings(logLines)
%E_MODERN_STRINGS Parse experiment log lines with string arrays.
%   report = e_modern_strings(logLines) takes a string array of log lines
%   such as "2026-03-14 10:02:11 run=42 status=OK value=3.7" and returns a
%   table with one row per line.

arguments
    logLines (:,1) string
end

% Pattern objects describe the fields we need
datePat = digitsPattern(4) + "-" + digitsPattern(2) + "-" + digitsPattern(2);
runPat = "run=" + digitsPattern(1, Inf);

% Parse run numbers and dates
runText = extractBetween(logLines, "run=", " ");
runId = str2double(runText);
dateText = extract(logLines, datePat);
when = datetime(dateText, 'InputFormat', 'yyyy-MM-dd');

% Status and value, found by splitting on spaces
parts = split(logLines, " ");
status = strings(numel(logLines), 1);
value = NaN(numel(logLines), 1);
for i = 1:numel(logLines)
    fields = parts(i, :);
    statusField = fields(startsWith(fields, "status="));
    if ~isempty(statusField)
        status(i) = extractAfter(statusField, "status=");
    end
    valueField = fields(startsWith(fields, "value="));
    if ~isempty(valueField)
        value(i) = str2double(extractAfter(valueField, "value="));
    end
end

% Clean up the status text
status = strip(lower(status));
status = replace(status, "ok", "pass");

% Build a human-readable message per run
msg = arrayfun(@(r, s) compose("Run %d finished with %s", r, s), ...
    runId, status);

% Flag the runs that need attention
failed = contains(status, "fail") | ismissing(status);
if any(failed)
    fprintf("%d of %d runs need attention\n", nnz(failed), numel(failed));
end

report = table(runId, when, status, value, msg, ...
    'VariableNames', ["run", "date", "status", "value", "message"]);

% Reporting helpers
header = join(["Run", "Date", "Status"], sprintf("\t"));
disp(header)

summaryLine = strjoin(string(unique(status)), ", ");
disp("Statuses seen: " + summaryLine)

% Double-quoted escapes
tab = "\t";
newline_ = "\n";
disp("Columns:" + tab + strlength(string(report.Properties.VariableNames)))
fprintf(newline_)

% Dates as strings for export
stamp = string(when, "yyyy/MM/dd");
disp(stamp(1))
end
