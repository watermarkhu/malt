function [val, cache] = memo_fib(n, cache)
%MEMO_FIB Fibonacci with a memo table passed in and out.
%   [val, cache] = MEMO_FIB(n) starts an empty cache.

if nargin < 2 || isempty(cache)
    cache = containers.Map('KeyType', 'double', 'ValueType', 'double');
end

if n < 0
    error('fib:negative', 'n must be >= 0, got %d', n);
end
if n <= 1
    val = n;
    return
end
if isKey(cache, n)
    val = cache(n);
    return
end
a = memo_fib(n - 1, cache);
b = memo_fib(n - 2, cache);
val = a + b;
cache(n) = val;
end

function out = fib_list(n)
out = zeros(1, n);
persistent calls
if isempty(calls)
    calls = 0;
end
for k = 1:n
    calls = calls + 1;
    out(k) = fib_nested(k);
end
fprintf('calls so far: %d\n', calls);

    function f = fib_nested(m)
        if m <= 2
            f = 1;
        else
            f = fib_nested(m - 1) + fib_nested(m - 2);
        end
    end
end

function r = safe_ratio(a, b)
r = a / b;
if isinf(r) || isnan(r)
    r = NaN;
end
end
