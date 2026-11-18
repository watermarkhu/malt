%GENERATE_NAMESPACE_FACTS Write the MATLAB facts of the committed fixture, as JSON.
%   Phase 1 of the MATLAB cross-check (tests/matlab_namespace.rs does phase 2). CI runs it with
%   matlab-actions/run-command, the licensed way to run MATLAB on a public repository:
%
%       run('crates/malt_cli/tests/matlab/generate_namespace_facts.m')
%
%   The test runs the same script when it has to start MATLAB itself. No arguments. Environment:
%     MALT_NAMESPACE_FACTS_OUT   the JSON file to write
%                                (default: <repo>/target/matlab/namespace_facts.json)
%     MALT_MATLAB_EXTRA_PATHS    an OS path list of local folders, added in front of the
%                                fixture, each with its plain subfolders (genpath)
%   The folders are recorded in the file ("roots"), so the comparison reads them from there.

here = fileparts(mfilename('fullpath'));
repo = fileparts(fileparts(fileparts(fileparts(here))));
addpath(here, '-end');

% Run from a folder that holds no project names, so that which() sees only the path.
previousFolder = pwd;
restoreFolder = onCleanup(@() cd(previousFolder));
cd(tempdir);

project = fullfile(here, 'project');
front = {fullfile(project, 'shadow_a'), fullfile(project, 'shadow_b'), project};
back = {fullfile(project, 'shadow_c')};

extras = getenv('MALT_MATLAB_EXTRA_PATHS');
if ~isempty(extras)
    folders = strsplit(extras, pathsep);
    for k = 1:numel(folders)
        if isempty(folders{k})
            continue
        end
        if ~isfolder(folders{k})
            error('generate_namespace_facts:extra', 'not a folder: %s', folders{k});
        end
        parts = strsplit(genpath(folders{k}), pathsep);
        front = [front, parts(~cellfun(@isempty, parts))];
    end
end

out = getenv('MALT_NAMESPACE_FACTS_OUT');
if isempty(out)
    out = fullfile(repo, 'target', 'matlab', 'namespace_facts.json');
end
if ~isfolder(fileparts(out))
    mkdir(fileparts(out));
end

% matlab_facts reads its configuration from a file: write one for this run.
configFile = [tempname, '.json'];
restoreConfig = onCleanup(@() delete(configFile));
config = struct('out', out, 'front', {front}, 'back', {back});
fid = fopen(configFile, 'w', 'n', 'UTF-8');
fprintf(fid, '%s', jsonencode(config));
fclose(fid);

matlab_facts(configFile);
fprintf('namespace facts written to %s (%d folders on the front of the path)\n', out, numel(front));
