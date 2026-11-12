function out = k_tabs_mixed(data, thresh)
%K_TABS_MIXED  thresholds and counts, indented with a mix of tabs and spaces
%	The help text itself uses a tab after the percent sign.
%
%	out = k_tabs_mixed(data, thresh)

if nargin < 2
	thresh = 0.5;
end

out = struct('count', 0, 'kept', []);
	
for ii = 1:numel(data)
	if data(ii) > thresh
	    out.count = out.count + 1;
	    out.kept(end+1) = data(ii);
	else
  	    % below threshold, nothing to do
	end
end

	% ---- second block, tab-indented by a careless editor ----
	if isempty(out.kept)
		out.status = 'none';
	elseif numel(out.kept) < 3
	  out.status = 'few';
	else
	    out.status = 'many';
	end

	ratio = 0;
	if data(:).' * ones(numel(data),1) ~= 0
	    ratio = out.count / numel(data);
	end
	out.ratio = ratio;

	% helper values
	lo = min(data);
	hi = max(data);
	out.range = [lo, hi];
	out.mid   = (lo + hi) / 2;

	switch out.status
		case 'none'
			out.code = 0;
		case {'few', 'some'}
		    out.code = 1;
		otherwise
			out.code = 2;
	end

	while out.code < 2
		out.code = out.code + 1;
	end

	fprintf('count=%d status=%s\n', out.count, out.status);
	fprintf('ratio=%.3f mid=%.3f\n', out.ratio, out.mid);

	names = fieldnames(out);
	for k = 1:numel(names)
	    if isnumeric(out.(names{k})) && isscalar(out.(names{k}))
	        fprintf('%s = %g\n', names{k}, out.(names{k}));
	    end
	end

	tmp = clamp_values(out.kept, 0, 10);
	out.clamped = tmp;

end

function v = clamp_values(v, lo, hi)
	v = min(max(v, lo), hi);
	if isempty(v)
	    v = zeros(1, 0);
	end
end
