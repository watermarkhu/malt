function report = log_regexp(lines)
%LOG_REGEXP Pulls structured fields out of web server log lines.

pattern = '^(?<ip>\d+\.\d+\.\d+\.\d+) \S+ \S+ \[(?<when>[^\]]+)\] "(?<method>[A-Z]+) (?<path>\S+)[^"]*" (?<status>\d{3}) (?<bytes>\d+|-)';
report = struct('ip', {}, 'method', {}, 'path', {}, 'status', {});
bad = 0;
for i = 1:numel(lines)
    tok = regexp(lines{i}, pattern, 'names', 'once');
    if isempty(tok)
        bad = bad + 1;
        continue;
    end
    report(end+1) = struct('ip', tok.ip, 'method', tok.method, ...
        'path', tok.path, 'status', str2double(tok.status));
end
fprintf('%d lines, %d unparsed\n', numel(lines), bad);

statuses = [report.status];
errors = report(statuses >= 500);
if ~isempty(errors)
    paths = unique({errors.path});
    for p = 1:numel(paths)
        n = sum(strcmp({errors.path}, paths{p}));
        fprintf('%s: %d server errors\n', paths{p}, n);
    end
end

ips = string({report.ip});
counts = containers.Map('KeyType', 'char', 'ValueType', 'double');
for i = 1:numel(ips)
    k = char(ips(i));
    if isKey(counts, k)
        counts(k) = counts(k) + 1;
    else
        counts(k) = 1;
    end
end

q = "GET /api/v1/items?id=42";
parts = split(q, "?");
query = parts(2)
ver = regexp(q, 'v(\d+)', 'tokens');
if ~isempty(ver)
    apiVersion = str2double(ver{1}{1});
end
cleaned = regexprep({report.path}, '\?.*$', '');
strs = strrep(cleaned, '/', '|');
joined = strjoin(strs, ', ')
top = max(cell2mat(values(counts)));
end
