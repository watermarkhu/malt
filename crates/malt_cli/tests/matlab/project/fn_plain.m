function y = fn_plain(a, b)
% FN_PLAIN Two inputs, one output, and a local function.
y = local_twice(a) + b;
end

function z = local_twice(x)
z = 2 * x;
end
