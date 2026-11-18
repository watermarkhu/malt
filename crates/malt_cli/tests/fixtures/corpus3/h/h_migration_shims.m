function varargout = h_migration_shims(op, varargin)
%H_MIGRATION_SHIMS Compatibility wrappers used during the old -> new release move
%   [VARARGOUT] = H_MIGRATION_SHIMS(OP, ARGS...) dispatches to the shim named OP.
%   Remove once the minimum supported release is raised.
%
%   Ops: 'datenum', 'strread', 'findstr', 'wavread', 'version', 'nargchk', 'tempdir'

if isunix
    sep = '/';
elseif ispc
    sep = '\';
else
    sep = filesep;
end

if nargin < 1
    error('h_migration_shims:noOp', 'Give an operation name');
end

switch lower(op)
    case 'datenum'
        varargout{1} = shimDatenum(varargin{:});
    case 'strread'
        varargout{1} = shimStrread(varargin{:});
    case 'findstr'
        varargout{1} = shimFindstr(varargin{:});
    case 'wavread'
        [y, fs] = shimWavread(varargin{:});
        varargout = {y, fs};
    case 'version'
        varargout{1} = shimVersion();
    case 'nargchk'
        varargout{1} = shimNargchk(varargin{:});
    case 'tempdir'
        varargout{1} = shimTempdir(sep);
    otherwise
        error('h_migration_shims:unknownOp', 'No shim for %s', op);
end
end

function d = shimDatenum(varargin)
% datetime objects need the conversion, the old datenum does not accept them
if verLessThan('matlab', '9.1')
    d = datenum(varargin{:});
elseif nargin == 1 && isa(varargin{1}, 'datetime')
    d = datenum(varargin{1});
else
    d = datenum(varargin{:});
end
end

function c = shimStrread(str, fmt, varargin)
if exist('strread', 'file') == 2 && ~verLessThan('matlab', '8.0')
    warning('off', 'MATLAB:strread:obsolete');
    c = strread(str, fmt, varargin{:});
    warning('on', 'MATLAB:strread:obsolete');
else
    c = textscan(str, fmt, varargin{:});
end
end

function idx = shimFindstr(a, b)
% findstr was removed, strfind has the same result for these inputs
if numel(a) > numel(b)
    t = a;
    a = b;
    b = t;
end
idx = strfind(b, a);
end

function [y, fs] = shimWavread(file, varargin)
if exist('audioread', 'file') == 2
    [y, fs] = audioread(file, varargin{:});
elseif exist('wavread', 'file') == 2
    [y, fs] = wavread(file, varargin{:});
else
    error('h_migration_shims:noReader', 'No audio reader on this path');
end
end

function s = shimVersion()
v = ver('matlab');
s = struct('release', v.Release, 'version', version, ...
    'isOld', verLessThan('matlab', '9.10'));
end

function msg = shimNargchk(lo, hi, n)
% returns a message string like the old nargchk did, empty when fine
msg = '';
if n < lo
    msg = 'Not enough input arguments.';
elseif n > hi
    msg = 'Too many input arguments.';
end
end

function d = shimTempdir(sep)
% tempdir is fine on all platforms, the TMP fallback is for an old Windows setup
d = tempdir;
if isempty(d)
    d = getenv('TMP');
end
if isempty(d) || d(end) ~= sep
    d = [d sep];
end
end
