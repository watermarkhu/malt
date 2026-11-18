function out = h_args_block(data, opts)
arguments
    data (:,:) double
    opts.Window (1,1) {mustBePositive, mustBeInteger} = 3
    opts.Method (1,1) string {mustBeMember(opts.Method, ["mean", "median"])} = "mean"
    opts.Normalize (1,1) logical = false
end
n = size(data, 1);
out = zeros(n, size(data, 2));
for r = 1:n
    lo = max(1, r - opts.Window);
    hi = min(n, r + opts.Window);
    block = data(lo:hi, :);
    switch opts.Method
        case "mean"
            out(r, :) = mean(block, 1);
        case "median"
            out(r, :) = median(block, 1);
    end
end
if opts.Normalize
    out = out ./ max(abs(out(:)));
end
end
