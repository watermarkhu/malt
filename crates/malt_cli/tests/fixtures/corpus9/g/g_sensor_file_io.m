% Reads a semicolon-separated sensor export, cleans it and writes a summary.
% The export has a two-line header and a free-text comment column.

inFile = fullfile(tempdir, 'sensor_export.csv');
outFile = fullfile(tempdir, 'sensor_summary.txt');

fid = fopen(inFile, 'r');
if fid == -1
    error('sensor:open', 'Cannot open %s', inFile);
end
header1 = fgetl(fid);
header2 = fgetl(fid);
rows = {};
lineNo = 2;
while true
    line = fgetl(fid);
    lineNo = lineNo + 1;
    if ~ischar(line)
        break
    end
    if isempty(strtrim(line)) || line(1) == '#'
        continue
    end
    rows{end+1} = line;
end
fclose(fid);

nRows = numel(rows);
readings = nan(nRows, 3);
comments = cell(nRows, 1);
for r = 1:nRows
    f = strsplit(rows{r}, ';');
    if numel(f) < 4
        comments{r} = 'short row';
        continue
    end
    readings(r, 1) = str2double(f{2});
    readings(r, 2) = str2double(f{3});
    readings(r, 3) = str2double(f{4});
    if numel(f) >= 5
        comments{r} = strtrim(f{5});
    end
end

valid = ~any(isnan(readings), 2);
clean = readings(valid, :);
fprintf('kept %d of %d rows\n', size(clean, 1), nRows);

try
    fout = fopen(outFile, 'w');
    fprintf(fout, 'mean temp %.2f\n', mean(clean(:, 1)));
    fprintf(fout, 'max humidity %.1f\n', max(clean(:, 2)));
    fprintf(fout, 'min pressure %.1f\n', min(clean(:, 3)));
    fclose(fout);
catch ME
    fclose('all');
    rethrow(ME)
end

summary = readtable(outFile, 'ReadVariableNames', false);
tmp = 1;
emptyComments = sum(cellfun(@isempty, comments));
clear header1
