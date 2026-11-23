% inventory tool for the shop -- v3 (v2 was lost in a crash)
clear all
close all
clc

DBFILE = 'inventory.txt';
items = {};
qty = [];
running = 1;
if exist(DBFILE, 'file') == 2
    fid = fopen(DBFILE, 'r');
    while ~feof(fid)
        line = fgetl(fid);
        if ischar(line)
            parts = strsplit(line, ',');
            items{end+1} = parts{1};
            qty(end+1) = str2double(parts{2});
        end
    end
    fclose(fid);
else
    disp('no database yet, starting empty')
end

while 1
    choice = menu('Inventory', 'Add item', 'Show', 'Remove', 'Save', 'Quit');
    switch choice
        case 1
            name = input('name: ', 's');
            q = input('quantity: ');
            if ~isnumeric(q) || isempty(q)
                msgbox('bad number', 'error')
                continue
            end
            items{end+1} = name;
            qty(end+1) = q;
        case 2
            for k = 1:length(items)
                fprintf('%d) %s x%d\n', k, items{k}, qty(k))
            end
            if isempty(items)
                disp('nothing in stock')
            end
        case 3
            idx = input('index to remove: ');
            if idx < 1 | idx > length(items)
                error('index out of range: %d', idx)
            end
            items(idx) = [];
            qty(idx) = [];
        case 4
            fid = fopen(DBFILE, 'w');
            for k = 1:length(items)
                fprintf(fid, '%s,%d\n', items{k}, qty(k));
            end
            fclose(fid);
            disp('saved')
        case 5
            break
        otherwise
            disp('??')
    end
end

% total units in stock, printed after quitting
if exist('qty', 'var') ~= 1
    total = 0;
else
    total = sum(qty);
end
disp(['total units: ' num2str(total)])

% low stock warning, magic number 3
for k = 1:length(qty)
    if qty(k) < 3
        fprintf('LOW: %s (%d)\n', items{k}, qty(k));
    end
end

% search by name, old style
q = input('search for: ', 's');
found = 0;
for k = 1:length(items)
    if strcmpi(items{k}, q)
        found = k;
    end
end
if found == 0
    msgbox(['not found: ' q])
else
    fprintf('found at %d\n', found)
end

% something copied from the add branch, slightly edited
n = input('how many to add: ');
if ~isnumeric(n)
    error('need a number')
end
if n > 0
    qty = qty + n;
end

% commented out old menu
% choice = input('1 add, 2 show: ');

% bad check, left over
if running == 1 & isempty(items) == 0
    disp('ok')
end
