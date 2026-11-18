function out = k_modern_mix(tbl, opts)
% K_MODERN_MIX  table, string and datetime work with sloppy formatting
arguments
    tbl table
    opts.Start (1,1) datetime = datetime(2024,1,1)
    opts.Label string = "report"
    opts.Scale (1,1) double {mustBePositive} = 1
    opts.Tags (1,:) string = ["a","b";"c","d"]
end

if ~istable(tbl) , error('k:notTable', 'input must be a table'); end

names = string(tbl.Properties.VariableNames);
d = opts.Start + days(0:height(tbl)-1)';
tbl.Date = d(:);
tbl.Value = tbl.Value.*opts.Scale;

label = opts.Label + " :: " + string(datestr(opts.Start,'yyyy-mm-dd'));
summary = sprintf("%s rows=%d", label, height(tbl));
slug = regexprep(summary, '\s+', '_');

% a map used as a lookup table
lookup = containers.Map();
lookup('a') = 1;
lookup("b") = 2;
lookup_keys = keys(lookup);
lookup_vals = values(lookup);

% masks and subsets
mask = tbl.Value > 0 & ~isnan(tbl.Value);
sub = tbl(mask, :);
out = sortrows(sub, 'Date', 'descend');

% grouped summary using findgroups
g = findgroups(out.Category);
avg = splitapply(@mean, out.Value, g);
groupNames = splitapply(@(c) c(1), out.Category, g);

% string handling, mixed quote styles
s = strjoin(names, ", ");
greeting = "Hello, " + opts.Label;
msg = 'it''s fine';
t = strtrim(sprintf('  %s  ', slug));
parts = split(summary, "=");
upperParts = upper(parts);
hasRows = contains(summary, "rows");
startsOk = startsWith(summary, opts.Label);

% datetime arithmetic and formatting
endDate = max(out.Date);
span = endDate - opts.Start;
spanDays = days(span);
weekdayName = string(weekday(endDate));
monthText = string(month(endDate, 'name'));
dt2 = datetime('2025-03-01', 'InputFormat', 'yyyy-MM-dd');
delta = between(opts.Start, dt2, 'days');

% a little helper, local to the file
out.Summary = repmat(summary, height(out), 1);
out.Group = groupNames;
out.Avg = avg;
out.Note = greeting;
out.Msg = msg;
out.Tags = opts.Tags;
out.Flags = [hasRows, startsOk];
out.Span = spanDays;
out.Delta = delta;
out.Keys = lookup_keys;
out.Vals = lookup_vals;
out.Weekday = weekdayName;
out.Month = monthText;
out.Parts = upperParts;
out.Joined = s;
out.Stamp = normalise_stamp(out.Date);

end

function st = normalise_stamp(dates)
    st = string(dates, 'yyyyMMdd');
    st(ismissing(st)) = "unknown";
end
