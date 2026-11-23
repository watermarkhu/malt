function registry = plugin_registry(pluginDir)
%PLUGIN_REGISTRY Loads the plugin descriptors found in a folder.
%   Each descriptor is a small text file of name = value lines. The
%   values are evaluated, so the files must come from a trusted source.

if ~isfolder(pluginDir)
    error('plugins:noDir', 'Not a folder: %s', pluginDir)
end
files = dir(fullfile(pluginDir, '*.cfg'));
registry = containers.Map();

for i = 1:length(files)
    fname = fullfile(files(i).folder, files(i).name);
    txt = fileread(fname);
    entries = regexp(txt, '(\w+)\s*=\s*([^\n]+)', 'tokens');
    desc = struct();
    for j = 1:numel(entries)
        key = entries{j}{1};
        valueText = entries{j}{2};
        try
            desc.(key) = eval(valueText);
        catch
            desc.(key) = valueText;
        end
    end
    if ~isfield(desc, 'name')
        desc.name = files(i).name(1:end-4);
    end
    registry(desc.name) = desc;
end

names = keys(registry);
for i = 1:numel(names)
    d = registry(names{i});
    if isfield(d, 'enabled') && d.enabled
        feval(@disp, ['loaded ' names{i}]);
    end
end

result = evalin('base', 'exist(''ans'', ''var'')');
f = str2func('@(x) x + 1');
val = f(41);
fn = 'magic';
M = feval(fn, 4);
registry('count') = numel(names);
end
