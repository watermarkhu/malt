% R3_TABLE_JOIN  Join an orders table to a customers table and summarise.
% Both tables are built here so the script runs without data files.

customers = table([1; 2; 3; 4], ...
    {'Ada'; 'Bo'; 'Cy'; 'Di'}, ...
    {'north'; 'south'; 'north'; 'west'}, ...
    'VariableNames', {'Id', 'Name', 'Region'});

orders = table([10; 11; 12; 13; 14], [1; 2; 1; 3; 9], ...
    [25.5; 13.0; 99.9; 5.25; 12.0], ...
    'VariableNames', {'OrderId', 'CustId', 'Amount'});

orders.Amount(orders.Amount < 10) = 10;

joined = outerjoin(orders, customers, 'LeftKeys', 'CustId', 'RightKeys', 'Id', ...
    'MergeKeys', true);

missing = joined(ismissing(joined.Name), :);
nMissing = height(missing)

byRegion = groupsummary(joined, 'Region', 'sum', 'Amount');
byRegion

sorted = sortrows(joined, {'Region', 'Amount'}, {'ascend', 'descend'});
top = sorted(1:min(2, height(sorted)), {'Name', 'Amount'})

totals = varfun(@sum, joined, 'InputVariables', 'Amount', 'GroupingVariables', 'Region');
disp(totals)

joined.Tier = repmat("std", height(joined), 1);
joined.Tier(joined.Amount > 50) = "gold";
tiers = unique(joined.Tier)

s = summary(joined.Amount);
avg = mean(joined.Amount);

csvName = fullfile(tempdir, 'joined.csv');
writetable(joined, csvName);
back = readtable(csvName);
delete(csvName)
nCols = width(back);
