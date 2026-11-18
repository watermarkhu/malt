function cfg = g_parse_opts(varargin)
%PARSE_OPTS  Parse name-value options with inputParser.
p = inputParser;
p.FunctionName = 'g_parse_opts';
addRequired(p, 'mode', @(s) any(strcmp(s, {'fast', 'slow'})));
addOptional(p, 'level', 1, @isnumeric);
addParameter(p, 'Tolerance', 1e-6, @(x) x > 0);
addParameter(p, 'Verbose', false, @islogical);
addParameter(p, 'Labels', {}, @iscellstr);
parse(p, varargin{:});
cfg = p.Results;
if cfg.Verbose
    fprintf('mode=%s level=%d\n', cfg.mode, cfg.level);
end
unused = p.UsingDefaults;
if ~isempty(find(strcmp(unused, 'Tolerance')))
    cfg.Tolerance = cfg.Tolerance * cfg.level;
end
names = fieldnames(cfg);
for i = 1:length(names)
    val = cfg.(names{i});
    if ischar(val)
        cfg.(names{i}) = strtrim(val);
    end
end
if length(cfg.Labels) == 0
    cfg.Labels = {'none'};
end
end
