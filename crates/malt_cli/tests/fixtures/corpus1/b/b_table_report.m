function report = b_table_report(orders, customers, startDate)
%B_TABLE_REPORT Build a monthly summary report from order and customer tables.
%   REPORT = B_TABLE_REPORT(ORDERS, CUSTOMERS) joins the two tables on
%   CustomerID, groups by region and month and returns a text report.
%
%   REPORT = B_TABLE_REPORT(ORDERS, CUSTOMERS, STARTDATE) ignores orders
%   placed before STARTDATE (default 1 January 2026).

arguments
    orders table
    customers table
    startDate datetime = datetime(2026, 1, 1)
end

% Join orders to customers on the shared key
joined = join(orders, customers, 'Keys', 'CustomerID');

% Keep only recent orders and derive a month column
keep = joined.OrderDate >= startDate;
joined = joined(keep, :);
joined.Month  = dateshift(joined.OrderDate, 'start', 'month');
joined.Region = categorical(joined.Region);

% Total and order count per region and month
summary = groupsummary(joined, {'Region', 'Month'}, 'sum', 'Amount');
summary = renamevars(summary, 'sum_Amount', 'Total');
summary = sortrows(summary, {'Region', 'Month'});

% Derived columns
summary.Share = summary.Total ./ sum(summary.Total);
summary.Days  = days(summary.Month - startDate);

% Per-region order counts with varfun
regionCounts = varfun(@numel, joined, ...
    'InputVariables', 'Amount', ...
    'GroupingVariables', 'Region');

% Weekly totals through a timetable
tt = table2timetable(joined(:, {'OrderDate', 'Amount'}), ...
    'RowTimes', 'OrderDate');
weekly = retime(tt, 'weekly', 'sum');

% Short region codes from a Map
codes = containers.Map({'North', 'South', 'East', 'West'}, ...
    {'N', 'S', 'E', 'W'});

lines = strings(height(summary), 1);
for r = 1:height(summary)
    regionKey = char(summary.Region(r));
    if isKey(codes, regionKey)
        code = codes(regionKey);
    else
        code = '?';
    end
    lines(r) = compose("%s %s %10.2f (%5.1f%%)", code, ...
        string(summary.Month(r), 'yyyy-MM'), ...
        summary.Total(r), 100 * summary.Share(r));
end

% Header and footer lines built with sprintf
header = sprintf('%-6s %-8s %10s %8s', 'Region', 'Month', 'Total', 'Share');
footer = sprintf('%d regions, %d months, %d weekly buckets', ...
    numel(unique(summary.Region)), numel(unique(summary.Month)), ...
    height(weekly));

report = strjoin([header; lines; footer], newline);

% Warn when a region has no orders in the period
if height(regionCounts) < numel(categories(joined.Region))
    warning('b_table_report:emptyRegion', ...
        'Some regions have no orders since %s.', char(startDate));
end

end
