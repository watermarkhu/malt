function [y, stats] = c_crlf_tabs(x, gain)
%C_CRLF_TABS Scale, clip and summarise a signal.
%   [Y, STATS] = C_CRLF_TABS(X, GAIN) multiplies X by GAIN and clips the
%   result to the range set by the sensor limits.

	if nargin < 2
		gain = 1.0;
	end
	limits = [-5 5];

	% Apply the gain and clip
	y = gain * x(:);
	y(y < limits(1)) = limits(1);
	y(y > limits(2)) = limits(2);

	% Count how many samples hit each limit
	nLow = sum(y == limits(1));
	nHigh = sum(y == limits(2));
	nClipped = nLow + nHigh;

	stats = struct();
	stats.mean = mean(y);
	stats.std = std(y);
	stats.clipped = nClipped;
	stats.fraction = nClipped / max(numel(y), 1);

	% Running peak using a loop with a plain accumulator
	peak = 0;
	for k = 1:numel(y)
		if abs(y(k)) > peak
			peak = abs(y(k));
		end
	end
	stats.peak = peak;

	% Reshape into two channels when the length allows it
	if mod(numel(y), 2) == 0
		z = reshape(y, 2, []);
		stats.channelMeans = mean(z, 2);
	else
		stats.channelMeans = [];
	end

	if stats.fraction > 0.2
		warning('c_crlf_tabs:clipping', '%.0f%% of samples clipped', 100 * stats.fraction);
	end

	y = reshape(y, size(x));
end
