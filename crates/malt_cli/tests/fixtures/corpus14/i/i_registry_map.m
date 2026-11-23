% i_registry_map: containers.Map based lookup with callbacks
reg = containers.Map();
reg('square') = @(x) x.^2;
reg('cube') = @(x) x.^3;
reg('neg') = @(x) -x;
keysList = keys(reg);
for k = 1:numel(keysList)
    f = reg(keysList{k});
    fprintf('%s(3) = %d\n', keysList{k}, f(3));
end
counts = containers.Map('KeyType', 'char', 'ValueType', 'double');
words = strsplit('the cat and the dog and the bird', ' ');
for i = 1:numel(words)
    w = words{i};
    if isKey(counts, w)
        counts(w) = counts(w) + 1;
    else
        counts(w) = 1;
    end
end
ks = keys(counts);
vs = cell2mat(values(counts));
[~, order] = sort(vs, 'descend');
for i = order
    disp([ks{i} ': ' num2str(vs(i))]);
end
if reg.isKey('square') && ~reg.isKey('quad')
    reg('quad') = @(x) x.^4;
end
total = 0;
for k = reg.keys
    fn = reg(k{1});
    total = total + fn(2);
end
total
