function files = h_report_builder(dataFile, outDir)
%H_REPORT_BUILDER Build CSV, text and figure report from a measurement log
%   FILES = H_REPORT_BUILDER(DATAFILE) writes into ./reports
%   FILES = H_REPORT_BUILDER(DATAFILE, OUTDIR)
%
%   DATAFILE is a CSV with a header row; first column is the row label.

if nargin < 2 || isempty(outDir)
    outDir = fullfile(pwd, 'reports');
end
if ~isfolder(outDir)
    mkdir(outDir);
end

stamp = datestr(now, 'yyyymmdd_HHMMSS');

% read the whole file, skip blank lines
fid = fopen(dataFile, 'r');
if fid == -1
    error('h_report_builder:open', 'cannot open %s', dataFile);
end
header = fgetl(fid);
cols = strsplit(header, ',');
rows = {};
while true
    ln = fgetl(fid);
    if ~ischar(ln)
        break;
    end
    if isempty(strtrim(ln))
        continue
    end
    rows{end+1, 1} = ln; %#ok<AGROW>
end
fclose(fid);

% parse numbers, first column is text
nRows = numel(rows);
M = zeros(nRows, numel(cols) - 1);
labels = cell(nRows, 1);
for r = 1:nRows
    parts = strsplit(rows{r}, ',');
    labels{r} = parts{1};
    for c = 2:numel(cols)
        M(r, c-1) = str2double(parts{c});
    end
end

T = array2table(M, 'VariableNames', matlab.lang.makeValidName(cols(2:end)));
T.Label = labels;
T = movevars(T, 'Label', 'Before', 1);

csvName = fullfile(outDir, sprintf('summary_%s.csv', stamp));
writetable(T, csvName);

% plain text summary
txtName = fullfile(outDir, sprintf('summary_%s.txt', stamp));
fid = fopen(txtName, 'w');
fprintf(fid, '%s\n', repmat('=', 1, 60));
fprintf(fid, 'Report generated %s\n', datestr(now));
fprintf(fid, '%s\n', repmat('-', 1, 60));
for c = 1:width(T)
    v = T{:, c};
    if isnumeric(v)
        fprintf(fid, '%-20s mean=%10.4f  std=%10.4f  max=%10.4f\n', ...
            T.Properties.VariableNames{c}, mean(v, 'omitnan'), std(v, 'omitnan'), max(v));
    end
end
fprintf(fid, '%s\n', repmat('=', 1, 60));
for r = 1:nRows
    fprintf(fid, '%s\n', formatRow(labels{r}, M(r, :)));
end
fclose(fid);

% figure: bar of the first data column, exported three ways
f = figure('Visible', 'off');
ax = axes(f); %#ok<LAXES>
bar(ax, categorical(labels), M(:, 1));
title(ax, 'Column 1 by label');
ylabel(ax, cols{2});
figName = fullfile(outDir, sprintf('fig_%s.png', stamp));
exportgraphics(f, figName, 'Resolution', 150);
saveas(f, fullfile(outDir, sprintf('fig_%s.fig', stamp)));
print(f, '-dpdf', fullfile(outDir, sprintf('fig_%s.pdf', stamp)));
close(f);

% banner and error count
banner = sprintf('%s | rows=%d | cols=%d | %5.1f%%', upper(dataFile), nRows, numel(cols), 100*nRows/max(nRows,1));
clean = regexprep(banner, '\s{2,}', ' ');
lines = strjoin({clean, repmat('-', 1, 10)}, newline);
idx = cellfun(@(s) ~isempty(regexp(s, '^ERR', 'once')), rows);
nErr = sum(idx);
fprintf('%s\nerrors: %d\n', lines, nErr);

files = {csvName, txtName, figName};
end

function s = formatRow(label, vals)
s = sprintf('%-12s', label);
for k = 1:numel(vals)
    s = [s sprintf(' %8.3e', vals(k))]; %#ok<AGROW>
end
end
