function y = scale(x, factor)
% SCALE Multiply x by a factor.
%   The factor defaults to two.
arguments
    x (1,1) double
    factor (1,1) double = 2
end
y = x * factor;
end
