function varargout = i_cell_utils(c, op)
if nargin < 2, op = 'len'; end
switch op
    case {'len', 'length'}
        varargout{1} = cellfun(@length, c);
    case 'upper'
        varargout{1} = cellfun(@upper, c, 'UniformOutput', false);
    case {'split', 'tok'}
        parts = cellfun(@(s) strsplit(s, ','), c, 'UniformOutput', false);
        varargout{1} = parts;
        if nargout > 1
            varargout{2} = cellfun(@numel, parts);
        end
    case 'join'
        s = '';
        for i = 1:numel(c)
            s = [s c{i}];
            if i < numel(c)
                s = [s ', '];
            end
        end
        varargout{1} = s;
    otherwise
        error('i_cell_utils:op', 'unknown op %s', op);
end
mask = cellfun(@isempty, c);
c(mask) = [];
idx = cellfun(@(s) ~isempty(strfind(s, 'a')), c);
A = arrayfun(@(k) k^2, 1:numel(c));
disp(A)
end
