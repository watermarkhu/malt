function y = j_scale_args(x, opts)
% R1_SCALE_ARGS  Scale a numeric array with options validated by an arguments block.

arguments
    x (:,:) double {mustBeFinite}
    opts.Method (1,1) string {mustBeMember(opts.Method, ["linear", "log", "sqrt"])} = "linear"
    opts.Offset (1,1) double = 0
    opts.Clip (1,2) double = [-Inf Inf]
end

switch opts.Method
    case "linear"
        y = x;
    case "log"
        y = log10(max(x, eps));
    case {"sqrt", "root"}
        y = sqrt(abs(x));
    otherwise
        error('r1:scale:method', 'Unknown method %s', opts.Method);
end

y = y + opts.Offset;
y = min(max(y, opts.Clip(1)), opts.Clip(2));

try
    s = size(y);
    if numel(s) > 2
        y = reshape(y, s(1), []);
    end
catch err
    rethrow(err);
end

means = mean(y, 2)';
counts = zeros(1, size(y, 1));
for r = 1:size(y, 1)
    counts(r) = nnz(y(r, :) > 0);
end

if all(counts == 0)
    warning('r1:scale:allZero', 'No positive values.')
elseif any(counts > 10)
    disp('dense rows')
end

sc = y(:, end:-1:1);
t = y';
w = y(end, :);
keep = y(:) ~= 0;
z = y(keep);
q = numel(z);
res.values = z;
res.n = q;
end
