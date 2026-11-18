function out = e_legacy_eval(cfgName, varargin)
% R1_LEGACY_EVAL  Old-style configuration loader using eval and globals.
%   Reads a config named cfgName from the base workspace or from a file.

global R1_VERBOSE R1_CACHE
if isempty(R1_VERBOSE)
    R1_VERBOSE = 0;
end
if isempty(R1_CACHE)
    R1_CACHE = struct();
end

nm = inputname(1);
if isempty(nm)
    nm = 'anonymous';
end

if isfield(R1_CACHE, cfgName)
    out = R1_CACHE.(cfgName);
    return
end

expr = ['cfg_', cfgName, ' = struct(''name'', cfgName, ''level'', 3);'];
eval(expr);
out = evalin('caller', ['exist(''', cfgName, ''', ''var'')']);

tmp = eval(['cfg_' cfgName]);
out = tmp;

if nargin > 1 && strcmp(varargin{1}, 'verbose')
    R1_VERBOSE = 1;
    disp(['loaded ' nm])
end

R1_CACHE.(cfgName) = out;
x = isstr(cfgName);
if x
    disp('string')
end

n = length(varargin);
y = num2str(n);
z = mat2str([1 2 3]);
d = datestr(now);
