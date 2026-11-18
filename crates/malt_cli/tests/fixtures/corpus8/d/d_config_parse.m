function cfg = d_config_parse(txt)
%R4_CONFIG_PARSE Parse key = value lines with sections into a struct.
%   Lines starting with ; or # are comments; [name] starts a section.

lines = strsplit(txt, newline);
cfg = struct();
section = 'global';
for i = 1:numel(lines)
    line = strtrim(lines{i});
    if isempty(line) || any(line(1) == ';#')
        continue
    end
    tok = regexp(line, '^\[(?<name>[A-Za-z_]\w*)\]$', 'names');
    if ~isempty(tok)
        section = tok.name;
        if ~isfield(cfg, section)
            cfg.(section) = struct();
        end
        continue
    end
    kv = regexp(line, '^(?<key>\w+)\s*=\s*(?<val>.*)$', 'names', 'once');
    if isempty(kv)
        warning('r4:config:syntax', 'Line %d ignored: %s', i, line);
        continue
    end
    num = str2double(kv.val);
    if ~isnan(num)
        value = num;
    elseif any(strcmpi(kv.val, {'yes', 'no', 'true', 'false'}))
        value = any(strcmpi(kv.val, {'yes', 'true'}));
    else
        value = kv.val;
    end
    if strcmp(section, 'global')
        cfg.(kv.key) = value;
    else
        cfg.(section).(kv.key) = value;
    end
end

names = fieldnames(cfg);
summary = strjoin(cellfun(@(n) sprintf('%s:%d', n, numel(n)), names, 'UniformOutput', false), ', ');
disp(summary)
known = {'timeout', 'retries'};
missing = setdiff(known, names);
if ~isempty(missing)
    fprintf('missing keys: %s\n', strjoin(missing, ' '))
end
extra = cfg;
end
