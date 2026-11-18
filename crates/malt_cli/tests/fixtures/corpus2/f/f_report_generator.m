function outFile = f_report_generator(records, outFile)
%F_REPORT_GENERATOR Write a fixed-width text report from a struct array.
%   F_REPORT_GENERATOR(RECORDS) writes report.txt in the current folder.
%   RECORDS is a struct array with fields name, value and unit.
%
%   OUTFILE = F_REPORT_GENERATOR(RECORDS, OUTFILE) uses the given path.

if nargin < 2 || isempty(outFile)
    outFile = 'report.txt';
end

rule = repmat('-', 1, 40);
header = sprintf('%-20s %10s %8s', 'Name', 'Value', 'Unit');

fid = fopen(outFile, 'w');
if fid == -1
    error('f_report_generator:open', 'Cannot open %s for writing', outFile);
end

% Title block
fprintf(fid, '%s\n', rule);
fprintf(fid, '%s\n', header);
fprintf(fid, '%s\n', rule);

% One line per record
for k = 1:numel(records)
    r = records(k);
    fprintf(fid, '%-20s %10.2f %8s\n', r.name, r.value, r.unit);
end
fprintf(fid, '%s\n', rule);

% Summary lines with different numeric formats
fprintf(fid, 'Count: %d\n', numel(records));
fprintf(fid, 'Mean: %6.2f\n', mean([records.value]));
fprintf(fid, 'Generated %s\n', datestr(now, 'yyyy-mm-dd HH:MM'));

% Join all values into one comma-separated line
strs = cellfun(@num2str, {records.value}, 'UniformOutput', false);
fprintf(fid, 'Values: %s\n', strjoin(strs, ', '));
fprintf(fid, 'Matrix: %s\n', mat2str([records.value]));

% Tabular view in the command window
names = {records.name}';
vals = [records.value]';
T = table(names, vals, 'VariableNames', {'Name', 'Value'});
disp(T)

% Pull an ID and number out of names like "AB-123"
pattern = '(?<id>[A-Z]{2})-(?<num>\d+)';
for k = 1:numel(records)
    parts = regexp(records(k).name, pattern, 'names', 'once');
    if ~isempty(parts)
        fprintf(fid, 'ID %s has number %s\n', parts.id, parts.num);
    end
end

% Close the file and check that the close worked
status = fclose(fid);
if status ~= 0
    warning('f_report_generator:close', 'fclose failed for %s', outFile);
end

type(outFile)
end
