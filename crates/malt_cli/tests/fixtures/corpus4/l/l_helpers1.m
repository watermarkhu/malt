function y = l_helpers1(x)
%L_HELPERS1 Rescale numeric input to the unit interval.
%   Y = L_HELPERS1(X) maps the finite values of X to [0, 1] using the
%   overall minimum and maximum. A constant input maps to zeros, so the
%   function never divides by zero.
%
%   Example:
%       y = l_helpers1([2 4 6])   % returns [0 0.5 1]

arguments
    x double
end

lo = min(x(:));
hi = max(x(:));
span = hi - lo;
if span == 0
    y = zeros(size(x));
else
    y = safeDivide(x - lo, span);
end
end

function q = safeDivide(a, b)
% Element-wise division that treats a zero denominator as Inf-free.
if b == 0
    q = zeros(size(a));
    return
end
q = a ./ b;
end
