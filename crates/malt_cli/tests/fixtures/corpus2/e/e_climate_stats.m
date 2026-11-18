%% Climate statistics for a daily temperature record
% Loads a daily series, smooths it, fits a trend and summarises by season.
% The input is assumed to be a CSV with columns: date, tempC.

clearvars
close all

%% Load data
dataFile = fullfile('data', 'station_daily.csv');
opts = detectImportOptions(dataFile);
opts = setvartype(opts, 'date', 'datetime');
T = readtable(dataFile, opts);

t = T.date;
temp = T.tempC;

% Drop missing readings
ok = ~isnan(temp);
t = t(ok);
temp = temp(ok);
fprintf('Using %d of %d daily readings\n', numel(temp), height(T));

%% Smoothing
% 31-day centred moving mean
tempSmooth = movmean(temp, 31, 'omitnan');

%% Linear trend
% x in years since the first observation
x = years(t - t(1));
p = polyfit(x, temp, 1);
trend = polyval(p, x);
fprintf('Trend: %.3f C per decade\n', p(1) * 10);

figure('Name', 'Trend')
plot(t, temp, ':', 'Color', [0.6 0.6 0.6]); hold on
plot(t, tempSmooth, 'LineWidth', 1.5)
plot(t, trend, 'r--')
hold off
xlabel('Date'); ylabel('Temperature (C)')
legend('Daily', '31-day mean', 'Linear trend', 'Location', 'best')
grid on

%% Seasonal summary
month = month(t);
season = strings(size(month));
season(ismember(month, [12 1 2])) = "Winter";
season(ismember(month, 3:5)) = "Spring";
season(ismember(month, 6:8)) = "Summer";
season(ismember(month, 9:11)) = "Autumn";

seasons = ["Winter", "Spring", "Summer", "Autumn"];
[G, seasonName] = findgroups(season);
meanBySeason = accumarray(G, temp, [], @mean);
stdBySeason = accumarray(G, temp, [], @std);
nBySeason = accumarray(G, 1);

% Put the rows in calendar order
[~, order] = ismember(seasons, seasonName);
meanBySeason = meanBySeason(order);
stdBySeason = stdBySeason(order);
nBySeason = nBySeason(order);

%% Bar chart with error bars
figure('Name', 'Seasonal means')
bar(meanBySeason, 'FaceColor', [0.2 0.5 0.8])
hold on
errorbar(1:numel(seasons), meanBySeason, stdBySeason ./ sqrt(nBySeason), ...
    'k.', 'LineWidth', 1)
hold off
set(gca, 'XTickLabel', seasons)
ylabel('Mean temperature (C)')
title('Seasonal mean with standard error')

%% Summary table
summary = table(seasons', meanBySeason, stdBySeason, nBySeason, ...
    'VariableNames', {'Season', 'Mean', 'Std', 'N'});
disp(summary)

% Count how many years are fully covered
yearsCovered = unique(year(t));
for k = 1:length(yearsCovered)
    yr = yearsCovered(k);
    nDays = sum(year(t) == yr);
    if nDays < 330
        fprintf('Warning: %d has only %d days\n', yr, nDays);
    end
end
