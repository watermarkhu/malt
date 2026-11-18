function out = f_inout_overwritten(out, x)
% An input that is also an output: unused when every path overwrites it first.
out = x + 1;
end

function out = f_inout_kept(out, x)
if x
    out = 1;
end
end

function out = f_inout_read(out, x)
out = out + x;
end

function out = f_inout_empty(out, x) %#ok<INUSD>
end
