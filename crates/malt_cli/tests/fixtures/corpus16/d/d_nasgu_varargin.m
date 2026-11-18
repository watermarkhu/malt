function d_nasgu_varargin(varargin)
% A dead store to varargin is NASGU like any other variable.
if nargin > 1
    varargin = varargin{2:nargin};
end
disp(1);
end

function r = d_second(a, varargin)
varargin = {};
r = a;
end

function r = d_third(varargin)
r = numel(varargin);
varargin = {1};
end

function r = d_fourth(varargin)
r = 1;
varargin = 5;
disp(varargin);
end
