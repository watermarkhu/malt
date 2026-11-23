function data = j_data_import_old(fname, mode)
%J_DATA_IMPORT_OLD  Read a lab data file in one of several old formats.
%   DATA = J_DATA_IMPORT_OLD(FNAME) reads a whitespace separated file with
%   one header line.  DATA = J_DATA_IMPORT_OLD(FNAME, MODE) selects the
%   reader: 'text', 'fscanf', 'import' or 'ascii'.
%
%   Originally written for the 2002 instrument logger, reused since.

if nargin < 2
    mode = 'fscanf';
end
data = [];

if ~exist(fname, 'file')
    error(['file not found: ' fname])
end

switch mode
    case 'text'
        % textread with a header line to skip
        [names, vals] = textread(fname, '%s %f', 'headerlines', 1);
        data.names = names;
        data.vals = vals;

    case 'fscanf'
        fid = fopen(fname, 'r');
        if fid == -1
            error('could not open the file')
        end
        header = fgetl(fid);
        nlines = 0;
        while ~feof(fid)
            line = fgetl(fid);
            if ~ischar(line), break, end
            if isempty(line) | line(1) == '%'
                continue
            end
            nlines = nlines + 1;
        end
        frewind(fid);
        fgetl(fid);                      % skip the header again
        A = fscanf(fid, '%f', [3 inf]);
        fclose(fid);
        data = A';
        fprintf('read %d data lines, header was: %s\n', nlines, header)

    case 'import'
        % importdata, with the number of header lines given explicitly
        raw = importdata(fname, ' ', 2);
        if isstruct(raw)
            data = raw.data;
            if isfield(raw, 'textdata')
                labels = raw.textdata;
            end
        else
            data = raw;
        end

    case 'ascii'
        % old load -ascii, the whole file must be numeric
        load(fname, '-ascii');
        [pathstr, name] = fileparts(fname);
        data = eval(name);
        clear(name)

    otherwise
        error(['unknown mode: ' mode])
end

% clean up rows with NaN, the old logger writes -999 for missing
if isnumeric(data)
    bad = find(data == -999);
    data(bad) = NaN;
    rows_bad = [];
    for i = 1:size(data, 1)
        if any(isnan(data(i, :)))
            rows_bad = [rows_bad i];
        end
    end
    if ~isempty(rows_bad)
        fprintf('%d rows contain missing values\n', length(rows_bad))
    end
end

% convert the time column from a date string if the logger wrote one
if exist('labels', 'var')
    if ~isempty(findstr(labels{1}, '/'))
        dv = datevec(labels(2:end), 'dd/mm/yyyy HH:MM:SS');
        data = [datenum(dv) data];
    end
end

% numbers with a decimal comma from the european logger
if ischar(data)
    data = strrep(data, ',', '.');
    data = str2num(data);
end

% a second pass with sscanf over the raw text, for odd files
fid = fopen(fname);
txt = fread(fid, inf, 'char=>char')';
fclose(fid);
nums = sscanf(txt, '%f');
if length(nums) ~= numel(data) & strcmp(mode, 'fscanf')
    disp('number count differs between the two readers')
end

% csv variant the lab used later in the project
if ~isempty(strfind(fname, '.csv'))
    csvdata = csvread(fname, 1, 0);
    data = csvdata;
end

% write a copy in a simple binary form for the fast reloader
save([fname(1:end-4) '_copy.mat'], 'data')

% count the columns, handy when the header is missing
ncols = size(data, 2);
if ncols == 0
    warning('no columns read')
end

% let the user keep only the rows in a window
if nargin > 2
    lo = varargin_lo;
end
