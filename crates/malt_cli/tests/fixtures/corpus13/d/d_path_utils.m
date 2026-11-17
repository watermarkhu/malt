function out = path_utils(paths, mode)
%PATH_UTILS Normalises and filters file paths for a build manifest.
%   mode is 'short' (file names only), 'dirs' (folders only) or 'full'.

if nargin < 2
    mode = 'full';
end
out = {};
for k = 1:numel(paths)
    p = strrep(paths{k}, '\', '/');
    p = regexprep(p, '/+', '/');
    [folder, name, ext] = fileparts(p);
    if isempty(ext)
        continue
    end
    switch mode
        case 'short'
            out{end+1} = [name ext];
        case 'dirs'
            out{end+1} = folder;
        case 'full'
            out{end+1} = p;
        otherwise
            error('paths:mode', 'bad mode %s', mode);
    end
end
out = unique(out, 'stable');
depth = cellfun(@(s) numel(strfind(s, '/')), out);
[~, order] = sort(depth);
out = out(order);
joined = fullfile(out{:});
if ispc
    sep = ';';
else
    sep = ':';
end
searchPath = strjoin(out, sep);
tmp = 'scratch';
end
