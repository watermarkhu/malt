function joined = table_join(customersFile, ordersFile)
%TABLE_JOIN Joins customers and orders on the customer id and summarises spend.

customers = readtable(customersFile, 'TextType', 'string');
orders = readtable(ordersFile);

hasId = ismember(orders.CustomerId, customers.Id);
orders = orders(hasId, :);
orders.Total = orders.Quantity .* orders.UnitPrice;

[~, rowC] = ismember(orders.CustomerId, customers.Id);
orders.Region = customers.Region(rowC);
orders.Name = customers.Name(rowC);

byRegion = groupsummary(orders, 'Region', 'sum', 'Total');
disp(byRegion)

joined = innerjoin(customers, orders, 'LeftKeys', 'Id', 'RightKeys', 'CustomerId');
joined = sortrows(joined, {'Region', 'Total'}, {'ascend', 'descend'});
top = joined(1:min(height(joined), 5), :);

spend = varfun(@sum, orders, 'InputVariables', 'Total', 'GroupingVariables', 'Region');
spend.Properties.VariableNames{'sum_Total'} = 'Spend';
missing = setdiff(customers.Id, orders.CustomerId);
nMissing = numel(missing);

for i = 1:height(top)
    row = top(i, :);
    fprintf('%s %s %.2f\n', row.Name{1}, row.Region{1}, row.Total);
end
joined.Properties.Description = 'joined';
end
