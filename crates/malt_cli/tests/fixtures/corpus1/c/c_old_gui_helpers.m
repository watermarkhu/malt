function c_old_gui_helpers(action, varargin)
%C_OLD_GUI_HELPERS Legacy helpers used by the lab's launch menu.
%   C_OLD_GUI_HELPERS('menu') shows a menu, C_OLD_GUI_HELPERS('log', msg)
%   appends a line to the session log, C_OLD_GUI_HELPERS('stamp') prints
%   the current time together with the elapsed CPU time.

if nargin < 1
    action = 'menu';
end

switch lower(action)
    case 'menu'
        showMenu();
    case 'log'
        writeLog(varargin{:});
    case 'stamp'
        printStamp();
    otherwise
        warning('off', 'backtrace');
        warning('Unknown action: %s', action);
end

end

function showMenu()
% Show a menu of options and run the chosen one
choice = menu('Choose an action', 'Load data', 'Run analysis', 'Quit');
if choice == 0 || choice == 3
    return
end

if exist('dataset', 'var')
    fprintf('Dataset already loaded\n');
else
    fprintf('No dataset in memory\n');
end

% Ask before overwriting the base workspace
answer = questdlg('Overwrite the base workspace?', 'Confirm', 'Yes', 'No', 'No');
if strcmp(answer, 'Yes')
    evalin('base', 'clear dataset');
    assignin('base', 'lastAction', choice);
end

% Check a struct field if it exists in the caller
cfg = evalin('base', 'config');
if isfield(cfg, 'outputDir')
    disp(['Output folder: ' cfg.outputDir])
end

end

function writeLog(msg, level)
if nargin < 2
    level = 'info';
end
logFile = fullfile(tempdir, 'lab_session.log');
fid = fopen(logFile, 'a');
if fid < 0
    warning('Could not open %s', logFile);
    return
end
stamp = datestr(now, 'yyyy-mm-dd HH:MM:SS');
fprintf(fid, '[%s] %s: %s\n', stamp, upper(level), msg);
fclose(fid);
end

function printStamp()
t0 = clock;
tic
pause(0.05)
elapsed = toc;
cpuUsed = cputime;
line = [ 'Stamp at ' datestr(now) ' wall ' num2str(elapsed) ' s, cpu ' num2str(cpuUsed) ' s' ];
disp(line)
fprintf('Clock: %d-%02d-%02d %02d:%02d\n', t0(1), t0(2), t0(3), t0(4), t0(5))

% Build a list of labels with indexes
labels = '';
for i = 1:3
    labels = [ labels 'run' num2str(i) ' ' ];
end
disp(strtrim(labels))
end
