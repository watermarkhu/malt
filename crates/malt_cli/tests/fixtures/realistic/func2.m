function varargout = func2(x, opts)
%FUNC2 Mixed features.
arguments
    x (:,1) double
    opts.Mode (1,1) string = "fast"
end
n = numel(x);
acc = zeros(n, 1);
parfor k = 1:n
    acc(k) = helper(x(k));
end
total = 0;
for k = 1:n
    if acc(k) > 0 && x(k) < 10
        total = total + acc(k);
    elseif isnan(acc(k))
        continue
    else
        break
    end
end
switch opts.Mode
    case "fast"
        scale = 1;
    case {"slow", "careful"}
        scale = 2;
    otherwise
        error('func2:mode', 'Unknown mode %s', opts.Mode);
end
f = @(t) t * scale + total;
varargout{1} = f(acc);
if nargout > 1
    varargout{2} = n;
end

    function y = helper(v)
        y = v^2 + scale_helper();
    end
end

function s = scale_helper()
    s = 1;
end
