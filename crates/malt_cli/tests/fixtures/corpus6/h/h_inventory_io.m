function inv = h_inventory_io(folder)
%R2_INVENTORY_IO Read stock files from a folder into a struct array.
%   Each .csv file has one header line and rows of: sku,qty,price

if nargin < 1
    folder = pwd;
end

files = dir(fullfile(folder, '*.csv'));
inv = struct('sku', {}, 'qty', {}, 'price', {}, 'file', {});

for f = 1:length(files)
    path = fullfile(folder, files(f).name);
    fid = fopen(path, 'rt');
    if fid < 0
        warning('r2:inv:open', 'Cannot read %s', path);
        continue
    end
    header = textscan(fid, '%s %s %s', 1, 'Delimiter', ',');
    data = textscan(fid, '%s %f %f', 'Delimiter', ',');
    fclose(fid);

    for r = 1:numel(data{1})
        item.sku = data{1}{r};
        item.qty = data{2}(r);
        item.price = data{3}(r);
        item.file = files(f).name;
        inv(end+1) = item;
    end
end

if isempty(inv)
    fprintf('no rows in %s\n', folder)
    return
end

[~, order] = sort({inv.sku});
inv = inv(order);

total = 0;
for k = 1:numel(inv)
    total = total + inv(k).qty * inv(k).price;
end
fprintf('stock value: %.2f\n', total)

lowStock = inv([inv.qty] < 5);
names = {lowStock.sku};
if ~isempty(names)
    disp(names)
end

prices = [inv.price];
avg = mean(prices)
inv(1).qty = int32(inv(1).qty);
spare = []
end
