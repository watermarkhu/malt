function cfg = h_config_loader(cfgFile, varargin)
%H_CONFIG_LOADER Load and validate a project configuration
%   CFG = H_CONFIG_LOADER(FILE) reads a JSON, INI or MAT file and merges defaults.
%   CFG = H_CONFIG_LOADER(FILE, 'Strict', true) errors on unknown keys.
%   CFG = H_CONFIG_LOADER(FILE, 'Defaults', S) extra defaults on top of the built-in ones.
%
%   Environment variable H_CFG_OVERRIDE (a path) takes precedence over FILE.

p = inputParser;
p.FunctionName = mfilename;
p.addRequired('cfgFile', @(s) ischar(s) || isstring(s));
p.addParameter('Strict', false, @islogical);
p.addParameter('Defaults', struct(), @isstruct);
p.parse(cfgFile, varargin{:});
opts = p.Results;

override = getenv('H_CFG_OVERRIDE');
if ~isempty(override)
    cfgFile = override;
end

if ~exist(cfgFile, 'file')
    % try relative to the config folder next to this file
    alt = fullfile(fileparts(mfilename('fullpath')), 'configs', cfgFile);
    if exist(alt, 'file') == 2
        cfgFile = alt;
    else
        error('h_config_loader:notFound', 'Config file not found: %s', cfgFile);
    end
end

[~, ~, ext] = fileparts(cfgFile);
switch lower(ext)
    case '.json'
        raw = fileread(cfgFile);
        cfg = jsondecode(raw);
    case '.ini'
        cfg = readIni(cfgFile);
    case '.mat'
        S = load(cfgFile);
        f = fieldnames(S);
        cfg = S.(f{1});
    otherwise
        warning('h_config_loader:unknownExt', 'Unknown extension "%s", using empty config', ext);
        cfg = struct();
end

defaults = struct('timeout', 30, 'retries', 2, 'logLevel', 'info', ...
    'paths', struct('data', 'data', 'out', 'out'));
defaults = mergeStruct(defaults, opts.Defaults);
cfg = mergeStruct(defaults, cfg);

validateattributes(cfg.timeout, {'numeric'}, {'scalar', 'positive'}, mfilename, 'timeout');
validateattributes(cfg.retries, {'numeric'}, {'scalar', 'integer', 'nonnegative'}, mfilename, 'retries');
if ~any(strcmp(cfg.logLevel, {'debug', 'info', 'warn', 'error'}))
    warning('h_config_loader:logLevel', 'logLevel "%s" unknown, using info', cfg.logLevel);
    cfg.logLevel = 'info';
end

if opts.Strict
    known = fieldnames(defaults);
    keys = fieldnames(cfg);
    for k = 1:numel(keys)
        if ~ismember(keys{k}, known)
            error('h_config_loader:unknownKey', 'Unknown key "%s"', keys{k});
        end
    end
end

% paths: data is relative to cwd, out is relative to the config file
cfg.paths.data = fullfile(cfg.paths.data);
cfg.paths.out = fullfile(fileparts(cfgFile), cfg.paths.out);
if ~isfolder(cfg.paths.out)
    mkdir(cfg.paths.out);
end
end

function out = mergeStruct(base, over)
% recursive merge, values in over win
out = base;
fn = fieldnames(over);
for k = 1:numel(fn)
    if isstruct(over.(fn{k})) && isfield(base, fn{k}) && isstruct(base.(fn{k}))
        out.(fn{k}) = mergeStruct(base.(fn{k}), over.(fn{k}));
    else
        out.(fn{k}) = over.(fn{k});
    end
end
end

function s = readIni(path)
s = struct();
fid = fopen(path, 'r');
if fid < 0
    error('h_config_loader:open', 'Cannot open %s', path);
end
c = onCleanup(@() fclose(fid));
section = 'root';
while ~feof(fid)
    line = strtrim(fgetl(fid));
    if isempty(line) || line(1) == ';' || line(1) == '#'
        continue
    end
    if line(1) == '['
        section = regexprep(line, '[\[\]]', '');
        s.(section) = struct();
        continue
    end
    eq = strfind(line, '=');
    if isempty(eq), continue, end
    key = strtrim(line(1:eq(1)-1));
    val = strtrim(line(eq(1)+1:end));
    num = str2double(val);
    if ~isnan(num)
        val = num;
    end
    if strcmp(section, 'root')
        s.(key) = val;
    else
        s.(section).(key) = val;
    end
end
end
