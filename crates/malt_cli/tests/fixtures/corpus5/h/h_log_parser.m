function entries = h_log_parser(logFile)
%R1_LOG_PARSER Parse a server log into a struct array.
%   Each line looks like: 2026-01-02 10:11:12 LEVEL [module] message

if nargin < 1, logFile = 'server.log'; end

entries = struct('time', {}, 'level', {}, 'module', {}, 'text', {});
pattern = '^(\S+ \S+) (\w+) \[(\w+)\] (.*)$';

fid = fopen(logFile);
if fid == -1
    entries = [];
    return;
end
c = onCleanup(@() fclose(fid));

lineNo = 0;
levels = containers.Map();
while true
    ln = fgetl(fid);
    if ln == -1
        break
    end
    lineNo = lineNo + 1;
    tok = regexp(ln, pattern, 'tokens', 'once');
    if isempty(tok)
        fprintf('skipping line %d\n', lineNo);
        continue;
    end
    e.time = tok{1};
    e.level = upper(tok{2});
    e.module = tok{3};
    e.text = tok{4};
    entries(end+1) = e; %#ok<AGROW>
    if isKey(levels, e.level)
        levels(e.level) = levels(e.level) + 1;
    else
        levels(e.level) = 1;
    end
end

counts = cellfun(@(k) levels(k), keys(levels));
names = keys(levels);
[~, idx] = max(counts);
fprintf('most common: %s\n', names{idx})

errs = entries(strcmp({entries.level}, 'ERROR'));
numErr = numel(errs)
lens = arrayfun(@(e) length(e.text), entries);
longest = max(lens)
end
