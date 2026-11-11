function out = e_validators(kind, value, varargin)
%E_VALIDATORS Reusable input checks and small parsers.
%   out = e_validators('positive', x) errors unless x > 0.
%   out = e_validators('filename', s) returns s as a char row.
%   out = e_validators('range', x, lo, hi) checks lo <= x <= hi.
%   out = e_validators('parsenum', s) parses a number or returns NaN.
%
%   Errors use identifiers starting with "e_validators:".

narginchk(2, 4);
nargoutchk(0, 1);

switch lower(kind)
    case 'positive'
        validateattributes(value, {'numeric'}, {'scalar', 'positive', 'real'}, ...
            mfilename, 'value');
        out = value;

    case 'filename'
        if isstring(value) && isscalar(value)
            value = char(value);
        end
        assert(ischar(value) && isrow(value), ...
            'e_validators:filename', 'File name must be a char row or scalar string.');
        out = value;

    case 'range'
        assert(numel(varargin) == 2, 'e_validators:range', ...
            'Range needs lower and upper bounds.');
        lo = varargin{1};
        hi = varargin{2};
        if lo > hi
            error('e_validators:badBounds', ...
                'Lower bound %g is above upper bound %g.', lo, hi);
        end
        out = all(value(:) >= lo & value(:) <= hi);

    case 'parsenum'
        out = parseNumber(value);

    otherwise
        error('e_validators:unknownKind', 'No validator called "%s".', kind);
end
end

function x = parseNumber(s)
% Accept numbers, numeric strings and string scalars.
if isnumeric(s)
    x = double(s);
    return
end

if isstring(s) || ischar(s)
    s = strtrim(char(s));
    x = str2double(s);
    if isnan(x) && ~strcmpi(s, 'nan')
        warning('e_validators:parseFailed', ...
            'Could not parse "%s" as a number; returning NaN.', s);
    end
else
    x = NaN;
end
end

function outName = nameOf(var) %#ok<DEFNU>
% Report the variable name passed in, useful for error messages.
outName = inputname(1);
if isempty(outName)
    outName = '<expression>';
end
end
