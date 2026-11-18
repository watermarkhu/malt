function [out, n] = fn_args(x, y)
% FN_ARGS A function with an arguments block: validated and defaulted inputs.
arguments
    x (1,1) double {mustBePositive}
    y (1,:) double = [1 2]
end
out = x;
n = numel(y);
end
