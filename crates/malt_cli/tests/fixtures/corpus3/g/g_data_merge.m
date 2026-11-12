% merge the three site files into one table  (T. Okafor)
clear; clc
% site files: site_A.txt (tab), site_B.csv, site_C.xls

fid = fopen('site_A.txt', 'r');
hdr = fgetl(fid);
C = textscan(fid, '%s %f %f', 'Delimiter', '\t');
fclose(fid);
idA = C{1};
valA = C{2};
dateA = C{3};

rawB = csvread('site_B.csv', 1, 0);
idB = cellstr(num2str(rawB(:,1)));
valB = rawB(:,2);
dateB = rawB(:,3);

[numC, txtC, rawC] = xlsread('site_C.xls');
idC = txtC(2:end, 1);
valC = numC(:, 2);
dateC = numC(:, 3);

% dates come in three formats, convert them all to datenum
d0 = datenum('2019-03-01', 'yyyy-mm-dd');
dateA = dateA + d0;
dateB = dateB + datenum('01-Mar-2019');

% stack everything
allid = [idA; idB; idC];
allval = [valA; valB; valC];
alldate = [dateA; dateB; dateC];
disp(['total rows: ' num2str(length(allid))])

% unique sites
sites = unique(allid);
nsites = length(sites);
fprintf('%d sites\n', nsites);

% which rows belong to site_A
[tf, loc] = ismember(idA, sites);
disp(sum(tf))

% sort by date then by value
M = sortrows([alldate allval], [1 -2]);
first = M(1,:)
last = M(end,:)

% per-site summary, the slow way
summary = zeros(nsites, 3);
for k = 1:nsites
    mask = strcmp(allid, sites{k});
    summary(k,1) = sum(mask);
    summary(k,2) = mean(allval(mask));
    summary(k,3) = max(alldate(mask));
end

% label strings built with concatenation
for k = 1:nsites
    label = ['site_' sites{k} ' n=' num2str(summary(k,1))];
    disp(label)
end

% write out the merged table
fid = fopen('merged.txt', 'w');
fprintf(fid, 'site,date,value\n');
for k = 1:size(M,1)
    fprintf(fid, '%s,%s,%.3f\n', sites{1}, datestr(M(k,1), 'yyyy-mm-dd'), M(k,2));
end
fclose(fid);

% same thing with sprintf into a big string, then written once
out = '';
for k = 1:size(M,1)
    out = [out sprintf('%s;%f\n', 'x', M(k,2))];
end
fid = fopen('merged2.txt', 'w');
fprintf(fid, '%s', out);
fclose(fid);

% dates as strings
dstr = datestr(alldate(1:5))
ymd = datevec(alldate(1));
yr = ymd(1)

% join and drop duplicates
[ud, iu] = unique(M(:,1));
M2 = M(iu, :);
size(M2)

% keep only rows with values in range, old style
keep = [];
for k = 1:size(M,1)
    if M(k,2) > 0 & M(k,2) < 100
        keep = [keep; M(k,:)];
    end
end
disp(size(keep,1))

% checking missing site names
missing = {};
for k = 1:length(idC)
    if isempty(find(strcmp(idA, idC{k})))
        missing{end+1} = idC{k};
    end
end

% monthly averages, counted by hand
months = zeros(12,1);
counts = zeros(12,1);
for k = 1:size(M,1)
    mo = str2double(datestr(M(k,1), 'mm'));
    months(mo) = months(mo) + M(k,2);
    counts(mo) = counts(mo) + 1;
end
avgM = months ./ max(counts, 1);
figure
bar(avgM)
xlabel('month')
ylabel('mean value')
% title('monthly averages') % check the axis label later
