function matlab_facts(configFile)
%MATLAB_FACTS Write what MATLAB says about the names of some folders, as JSON.
%   MATLAB_FACTS(CONFIG) reads CONFIG, a JSON file with the fields
%     out   - the JSON file to write (UTF-8)
%     front - folders put at the front of the path, in order (addpath -begin)
%     back  - folders put at the end of the path, in order (addpath -end)
%
%   The project code is not run. Names are found by listing the folders, then
%   resolved with which, read with meta.class.fromName, and their signatures
%   read with nargin and nargout. Nothing is constructed. The path is restored
%   afterwards. Each item has its own try/catch: a failure is recorded in
%   "errors" and the rest is written.
%
%   The output, consumed by crates/malt_cli/tests/matlab_namespace.rs:
%     names    - map of identifier to its facts (see FACT_FIELDS below)
%     what     - what(folder) for each listed folder, as MATLAB lists it
%     private  - the .m files of each private folder (not on the path)
%     skipped  - folders and files listed but not on the path (Contents.m, plain subfolders)
%     other    - .p and .mlx files, which MATLAB may prefer to .m
%     errors   - items that could not be read
%     roots    - the folders put on the path: front and back, as given in CONFIG

cfg = jsondecode(fileread(configFile));
front = to_cell(cfg.front);
back = to_cell(cfg.back);

oldPath = path();
restorePath = onCleanup(@() path(oldPath));
for k = numel(front):-1:1
    addpath(front{k}, '-begin');
end
for k = 1:numel(back)
    addpath(back{k}, '-end');
end

st = struct('idents', {{}}, 'what', {{}}, 'private', {{}}, 'skipped', {{}}, ...
    'other', {{}}, 'errors', {{}});
for k = 1:numel(front)
    st = walk_folder(front{k}, '', st);
end
for k = 1:numel(back)
    st = walk_folder(back{k}, '', st);
end

% One fact per identifier. The first listing wins for folders and packages
% (path order); files are resolved by which, as MATLAB does.
names = containers.Map('KeyType', 'char', 'ValueType', 'any');
for k = 1:numel(st.idents)
    item = st.idents{k};
    if isKey(names, item.ident)
        continue
    end
    try
        names(item.ident) = fact_of(item);
    catch err
        st.errors{end+1} = sprintf('%s: %s', item.ident, err.message);
    end
end

result = struct();
result.names = names;
result.what = st.what;
result.private = st.private;
result.skipped = st.skipped;
result.other = st.other;
result.errors = st.errors;
result.roots = struct('front', {front}, 'back', {back});

text = jsonencode(result, 'PrettyPrint', true);
fid = fopen(cfg.out, 'w', 'n', 'UTF-8');
if fid < 0
    error('matlab_facts:open', 'cannot write %s', cfg.out);
end
fprintf(fid, '%s\n', text);
fclose(fid);
end

function st = walk_folder(folder, prefix, st)
% Lists one folder on the path: its files, packages (+pkg) and class folders (@cls).
try
    w = what(folder);
    st.what{end+1} = struct('folder', folder, 'prefix', prefix, 'class', '', ...
        'm', {to_cell(w.m)}, 'classes', {to_cell(w.classes)}, ...
        'packages', {to_cell(w.packages)});
catch err
    st.errors{end+1} = sprintf('what %s: %s', folder, err.message);
end
entries = dir(folder);
for k = 1:numel(entries)
    name = entries(k).name;
    full = fullfile(folder, name);
    if strcmp(name, '.') || strcmp(name, '..') || startsWith(name, '.')
        continue
    end
    if entries(k).isdir
        if startsWith(name, '+')
            dotted = join_ident(prefix, name(2:end));
            st.idents{end+1} = struct('ident', ['+' dotted], 'kind', 'package', 'path', full);
            st = walk_folder(full, dotted, st);
        elseif startsWith(name, '@')
            ident = join_ident(prefix, name(2:end));
            st.idents{end+1} = struct('ident', ident, 'kind', 'class_folder', 'path', full);
            st = walk_class_folder(full, ident, st);
        elseif strcmp(name, 'private')
            files = dir(fullfile(full, '*.m'));
            for j = 1:numel(files)
                st.private{end+1} = struct('name', files(j).name(1:end-2), ...
                    'owner', folder, 'file', fullfile(full, files(j).name));
            end
        else
            st.skipped{end+1} = full;
        end
    else
        [~, stem, ext] = fileparts(name);
        if strcmp(ext, '.m') && ~strcmp(stem, 'Contents')
            st.idents{end+1} = struct('ident', join_ident(prefix, stem), 'kind', 'file', 'path', full);
        elseif strcmp(ext, '.m')
            st.skipped{end+1} = full;
        elseif any(strcmp(ext, {'.p', '.mlx'}))
            st.other{end+1} = full;
        end
    end
end
end

function st = walk_class_folder(folder, ident, st)
% A class folder: its methods are the files other than the class file.
[~, base] = fileparts(folder);
cls = base(2:end);
try
    w = what(folder);
    st.what{end+1} = struct('folder', folder, 'prefix', ident, 'class', cls, ...
        'm', {to_cell(w.m)}, 'classes', {to_cell(w.classes)}, ...
        'packages', {to_cell(w.packages)});
catch err
    st.errors{end+1} = sprintf('what %s: %s', folder, err.message);
end
files = dir(fullfile(folder, '*.m'));
for k = 1:numel(files)
    [~, stem] = fileparts(files(k).name);
    if ~strcmp(stem, cls)
        st.idents{end+1} = struct('ident', [ident '.' stem], 'kind', 'method', ...
            'path', fullfile(folder, files(k).name));
    end
end
end

function ident = join_ident(prefix, name)
% The identifier of NAME inside the package or class PREFIX ('' at the top).
if isempty(prefix)
    ident = name;
else
    ident = [prefix '.' name];
end
end

function fact = fact_of(item)
% The facts of one identifier. Fields not applicable are NaN (null in JSON).
fact = struct('kind', '', 'file', '', 'all', {{}}, 'nargin', NaN, 'nargout', NaN, ...
    'superclasses', {{}}, 'abstract', NaN, 'hidden', NaN, 'sealed', NaN, ...
    'dotted_resolves', NaN, ...
    'properties', containers.Map('KeyType', 'char', 'ValueType', 'any'), ...
    'methods', containers.Map('KeyType', 'char', 'ValueType', 'any'), ...
    'events', {{}}, 'enumeration', {{}});
switch item.kind
    case 'package'
        fact.kind = 'package';
        fact.file = item.path;
        return
    case 'class_folder'
        fact.kind = 'class_folder';
        fact.file = item.path;
        return
    case 'method'
        % An old-style method is resolved as @cls/method; its dotted name
        % (cls.method) is not a name MATLAB resolves, which is recorded.
        fact.kind = 'function';
        form = method_form(item.ident);
        fact.file = char(which(form));
        fact.all = to_cell(which(form, '-all'));
        fact.dotted_resolves = ~isempty(which(item.ident));
        return
end

% A file: which decides the file; the file's kind follows from what it is.
fact.file = char(which(item.ident));
fact.all = to_cell(which(item.ident, '-all'));
mc = [];
try
    mc = meta.class.fromName(item.ident);
catch
end
if ~isempty(mc)
    fact.kind = 'class';
    fact = fill_class_fields(fact, mc);
    return
end
try
    fact.nargin = nargin(item.ident);
    fact.nargout = nargout(item.ident);
    fact.kind = 'function';
catch
    fact.kind = 'script';
end
end

function fact = fill_class_fields(fact, mc)
% The class facts: bases, attributes, properties, methods, events, enumeration members.
fact.superclasses = names_of(mc.SuperclassList);
fact.abstract = logical(mc.Abstract);
fact.hidden = logical(mc.Hidden);
fact.sealed = logical(mc.Sealed);
props = containers.Map('KeyType', 'char', 'ValueType', 'any');
for p = reshape(mc.PropertyList, 1, [])
    if ~strcmp(p.DefiningClass.Name, mc.Name)
        continue
    end
    props(char(p.Name)) = struct('get_access', access_of(p.GetAccess), ...
        'set_access', access_of(p.SetAccess), 'constant', logical(p.Constant), ...
        'dependent', logical(p.Dependent), 'hidden', logical(p.Hidden), ...
        'abstract', logical(p.Abstract), 'get_method', ~isempty(p.GetMethod), ...
        'set_method', ~isempty(p.SetMethod));
end
fact.properties = props;
meths = containers.Map('KeyType', 'char', 'ValueType', 'any');
for m = reshape(mc.MethodList, 1, [])
    if ~strcmp(m.DefiningClass.Name, mc.Name)
        continue
    end
    meths(char(m.Name)) = struct('access', access_of(m.Access), ...
        'static', logical(m.Static), 'abstract', logical(m.Abstract), ...
        'hidden', logical(m.Hidden), 'inputs', {to_cell(m.InputNames)}, ...
        'outputs', {to_cell(m.OutputNames)});
end
fact.methods = meths;
fact.events = own_names(mc.EventList, mc.Name);
fact.enumeration = own_names(mc.EnumerationMemberList, mc.Name);
end

function c = names_of(list)
% The sorted names of a list of meta objects, superclasses for example.
c = sort(arrayfun(@(x) char(x.Name), reshape(list, 1, []), 'UniformOutput', false));
end

function c = own_names(list, className)
% The sorted names of the list members that this class defines.
c = {};
for k = 1:numel(list)
    if strcmp(list(k).DefiningClass.Name, className)
        c{end+1} = char(list(k).Name); %#ok<AGROW>
    end
end
c = sort(c);
end

function s = access_of(x)
% A member's access as text; a list of classes (restricted access) is 'restricted'.
if ischar(x) || isstring(x)
    s = char(x);
else
    s = 'restricted';
end
end

function form = method_form(ident)
% The name MATLAB resolves a method of a class folder by: 'cls.m' is '@cls/m',
% and 'pkg.cls.m' is 'pkg.@cls/m'.
dot = find(ident == '.', 1, 'last');
head = ident(1:dot-1);
parts = strsplit(head, '.');
form = ['@' parts{end} '/' ident(dot+1:end)];
if numel(parts) > 1
    form = [strjoin(parts(1:end-1), '.') '.' form];
end
end

function c = to_cell(x)
% A cell row of char from a cell, a string, a char or an empty value.
if isempty(x)
    c = {};
elseif ischar(x)
    c = {x};
elseif iscell(x)
    c = reshape(cellfun(@char, x, 'UniformOutput', false), 1, []);
else
    c = reshape(cellstr(string(x)), 1, []);
end
end
