function obj = oldcls(v)
% OLDCLS Old-style class constructor.
obj = class(struct('v', v), 'oldcls');
end
