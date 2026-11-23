function out = a_legacy_utils(varargin)
%A_LEGACY_UTILS Old-style helper kept for the batch scripts.
%   OUT = A_LEGACY_UTILS('name', value, ...) accepts these options:
%       'scale'   numeric scale factor (default 1)
%       'label'   text label (default 'untitled')
%       'mode'    'fast' or 'accurate' (default 'fast')
%
%   Uses a global CONFIG struct with the field CONFIG.verbose.

global CONFIG

if nargout > 1
    error('Too many output arguments')
end

scale = 1;
label = 'untitled';
mode = 'fast';

% Parse name/value pairs by hand
i = 1;
while i <= numel(varargin)
    key = varargin{i};
    switch lower(key)
        case 'scale'
            scale = varargin{i+1};
            i = i + 2;
        case 'label'
            label = varargin{i+1};
            i = i + 2;
        case 'mode'
            mode = varargin{i+1};
            i = i + 2;
        otherwise
            error('Unknown option: %s', key)
    end
end

if isempty(CONFIG)
    CONFIG.verbose = 0;
end

if findstr(label, 'test')
    disp('Test label, skipping extra work')
end

out = [];
for k = 1:10
    out = [out, k * scale];
end

if strcmpi(mode, 'accurate')
    out = out + 0.5;
end

if CONFIG.verbose
    disp(['Label: ' label ', mode: ' mode])
end

if exist('extra_data.mat', 'file') == 2
    load extra_data.mat
    out = out + extra_val;
end

tag = legacy_string(scale);
eval(sprintf('result_%s = out;', mode));
result = helper_scale(out, scale);
fprintf('%s: %s\n', label, tag)

end

function y = helper_scale(x, s)
% Multiply X by S.
y = x .* s;
end

function s = legacy_string(x)
% Format a number with num2str and drop the spaces.
s = num2str(x);
s = strrep(s, ' ', '');
end
