function report = deliberate_nasgu(data)
% A review of the data pipeline, with the dead stores a reviewer would ask about.

report = struct();
buffer = zeros(1, 100);
if isempty(data)
    buffer = [];
else
    buffer = data(:)';
end
report.size = numel(buffer);

pivot = max(buffer);
pivot = mean(buffer);
report.pivot = pivot;

temp = 1;
temp = 2;
temp = 3;
report.temp = temp;

cache = containers.Map();
cache = containers.Map('KeyType', 'char', 'ValueType', 'double');
report.keys = cache.Count;

flag = false;
for k = 1:numel(buffer)
    flag = buffer(k) > 0;
end
report.flag = flag;
other = buffer(1);
try
    other = str2double(data);
catch err
    other = NaN;
end
report.other = other;
if numel(data) > 1
    report.note = 'long';
end
end
