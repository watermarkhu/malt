function TT = d_csv_loader(filename, requiredCols)
%D_CSV_LOADER Read a price file into a timetable with cleaned columns.
%   TT = D_CSV_LOADER(FILENAME) reads a CSV with columns Date, Open, High,
%   Low, Close and Volume. Dates are parsed as datetimes, numeric columns
%   are converted to double and missing values are filled from the previous
%   row.
%
%   TT = D_CSV_LOADER(FILENAME, REQUIREDCOLS) overrides the list of columns
%   that must be present.

if nargin < 2
    requiredCols = {'Date', 'Open', 'High', 'Low', 'Close', 'Volume'};
end

if ~isfile(filename)
    error('d_csv_loader:notFound', 'File not found: %s', filename);
end

% Read as a table, keeping the header names exactly as they are in the file
opts = detectImportOptions(filename, 'VariableNamingRule', 'preserve');
opts = setvartype(opts, 'Date', 'char');
raw = readtable(filename, opts);

% Validate columns
missingCols = setdiff(requiredCols, raw.Properties.VariableNames, 'stable');
if ~isempty(missingCols)
    error('d_csv_loader:missingColumns', ...
        'Missing columns: %s', strjoin(missingCols, ', '));
end

% Parse dates; rows with unparseable dates are dropped
dates = datetime(raw.Date, 'InputFormat', 'yyyy-MM-dd', 'Format', 'yyyy-MM-dd');
bad = isnat(dates);
if any(bad)
    fprintf('Dropping %d rows with invalid dates.\n', nnz(bad));
end
raw = raw(~bad, :);
dates = dates(~bad);

% Numeric columns: convert text such as "1,234.5" and "n/a"
numCols = {'Open', 'High', 'Low', 'Close', 'Volume'};
data = zeros(height(raw), numel(numCols));
for k = 1:numel(numCols)
    col = raw.(numCols{k});
    if iscell(col) || isstring(col)
        col = str2double(strrep(string(col), ',', ''));
    end
    data(:, k) = double(col);
end

% Fill missing values from the previous observation
data = fillmissing(data, 'previous');

TT = array2timetable(data, 'RowTimes', dates, 'VariableNames', numCols);
TT = sortrows(TT);

% Sanity check: highs must not be below lows
nBad = sum(TT.High < TT.Low);
if nBad > 0
    warning('d_csv_loader:inconsistent', '%d rows have High < Low.', nBad);
end
end
