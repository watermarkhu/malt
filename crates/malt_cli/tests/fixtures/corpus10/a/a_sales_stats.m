% Monthly sales statistics for the regional report
% Uses the export in sales.csv (region, month, units, price)
data = readmatrix('sales.csv');
regions = data(:, 1);
months = data(:, 2);
units = data(:, 3);
price = data(:, 4)

revenue = units .* price;
totalByRegion = accumarray(regions, revenue)
avgByMonth = zeros(12, 1);
for m = 1:12
    sel = months == m;
    if any(sel)
        avgByMonth(m) = mean(units(sel));
    end
end

counts = histcounts(units, 0:50:500);
disp(counts)
smooth = movmean(avgByMonth, 3);
growth = diff(avgByMonth) ./ avgByMonth(1:end-1) * 100;
growth(isinf(growth)) = NaN;
bestMonth = find(avgByMonth == max(avgByMonth), 1)

figure
plot(1:12, avgByMonth, '-o')
hold on
plot(1:12, smooth, '--')
legend('raw', 'smoothed')
xlabel('month')
ylabel('units')
hold off

ranking = sortrows([regions, revenue], -2);
topThree = ranking(1:min(3, end), :);
fprintf('top region: %d\n', topThree(1, 1));

cumRevenue = cumsum(revenue);
half = cumRevenue(end) / 2;
firstHalf = find(cumRevenue >= half, 1)

if isempty(firstHalf)
    firstHalf = 0;
end
lowest = min(revenue(revenue > 0));
outliers = revenue(abs(revenue - mean(revenue)) > 3 * std(revenue))
nRegions = numel(unique(regions));
summary = struct('regions', nRegions, 'months', numel(unique(months)));
summary.total = sum(revenue);
summary
