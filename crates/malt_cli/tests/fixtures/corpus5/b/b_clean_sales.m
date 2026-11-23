% R1_CLEAN_SALES  Clean a sales export and print a summary.
% Written as a script on purpose: it reads a text file, fixes a few
% problems and prints totals. Run from the folder that holds sales.txt.

clc
format long
clear all

fid = fopen('sales.txt', 'r');
if fid < 0
    error('cannot open sales.txt');
end

header = fgetl(fid);
rows = {};
regions = {};
k = 0;
while ~feof(fid)
    line = fgetl(fid);
    if ~ischar(line) || isempty(strtrim(line))
        continue
    end
    k = k + 1;
    parts = strsplit(line, ',');
    rows{k} = parts;
    regions{end+1} = strtrim(parts{1});
end
fclose(fid);

amounts = zeros(1, k);
for i = 1:k
    raw = strrep(rows{i}{3}, '$', '');
    amounts(i) = str2double(raw);
end

bad = isnan(amounts)
amounts(bad) = [];
regions(bad) = [];

uniqueRegions = unique(regions);
totals = zeros(size(uniqueRegions));
for j = 1:numel(uniqueRegions)
    mask = strcmp(regions, uniqueRegions{j});
    totals(j) = sum(amounts(mask));
end

summary = table(uniqueRegions', totals', 'VariableNames', {'Region', 'Total'})

tmp = 0;
for q = 1:3
    tmp = tmp + q;
end

report = sprintf('%d rows, %d regions', k, numel(uniqueRegions));
disp(report)
fprintf('Grand total: %.2f\n', sum(totals));

if exist('old_totals.mat', 'file') == 2
    load old_totals.mat
    delta = totals - oldTotals;
end

result = []
for m = 1:numel(totals)
    result(end+1) = totals(m) * 1.1;
end
