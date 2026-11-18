function records = a_text_io(in_file, out_file)
%A_TEXT_IO Parse a log of sensor readings and write a summary.
%   RECORDS = A_TEXT_IO(IN_FILE, OUT_FILE) reads lines of the form
%       2026-01-05 12:00:01 sensor=A1 value=23.5 unit=C
%   into a struct array and writes a per-sensor summary to OUT_FILE.

if nargin < 2
    out_file = 'summary.txt';
end

pattern = ['^(?<date>\d{4}-\d{2}-\d{2}) (?<time>\S+) ' ...
           'sensor=(?<sensor>\w+) value=(?<value>[-\d.]+) unit=(?<unit>\w+)$'];

fid = fopen(in_file, 'r');
if fid < 0
    error('a_text_io:open', 'Cannot open %s', in_file)
end

records = struct('date', {}, 'time', {}, 'sensor', {}, 'value', {}, 'unit', {});
n_bad = 0;
line_no = 0;

% Read the file one line at a time
while ~feof(fid)
    line = fgetl(fid);
    line_no = line_no + 1;
    if ~ischar(line) || isempty(strtrim(line))
        continue
    end
    tok = regexp(line, pattern, 'names', 'once');
    if isempty(tok)
        n_bad = n_bad + 1;
        fprintf('Skipping line %d: %s\n', line_no, line);
        continue
    end
    rec.date = tok.date;
    rec.time = tok.time;
    rec.sensor = tok.sensor;
    rec.value = str2double(tok.value);
    rec.unit = tok.unit;
    records(end+1) = rec;
end
fclose(fid);

% Per-sensor summary
T = struct2table(records);
sensors = unique(T.sensor);

fid = fopen(out_file, 'w');
fprintf(fid, 'Sensor\tCount\tMean\tMax\n');
for s = 1:numel(sensors)
    rows = strcmp(T.sensor, sensors{s});
    vals = T.value(rows);
    fprintf(fid, '%s\t%d\t%.2f\t%.2f\n', sensors{s}, nnz(rows), mean(vals), max(vals));
end
fclose(fid);

fprintf('Parsed %d records, skipped %d lines.\n', numel(records), n_bad)
end
