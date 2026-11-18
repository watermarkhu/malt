function y = g_signal_chain(x, fs, varargin)
%R2_SIGNAL_CHAIN Detrend, filter and normalise a signal.
%   Optional name/value: 'Cutoff' (Hz), 'Order' (integer).

cutoff = 40;
order = 4;
for k = 1:2:numel(varargin)
    switch varargin{k}
        case 'Cutoff'
            cutoff = varargin{k+1};
        case 'Order'
            order = varargin{k+1};
        otherwise
            error('r2:chain:option', 'Unknown option %s', varargin{k});
    end
end

x = x(:);
t = (0:numel(x)-1)' / fs;
x = detrend(x);

wn = cutoff / (fs / 2);
[b, a] = butter(order, wn, 'low');
y = filtfilt(b, a, x);

energy = sum(y.^2) / numel(y);
rms_ = sqrt(energy)
peak = max(abs(y));
if peak > 0
    y = y / peak;
end

envelope = abs(hilbert(y));
spikes = find(diff(sign(y)) ~= 0);
nSpikes = numel(spikes)

windowLen = round(0.05 * fs);
starts = 1:windowLen:numel(y);
means = zeros(size(starts));
for w = 1:numel(starts)
    seg = y(starts(w):min(starts(w)+windowLen-1, end));
    means(w) = mean(seg);
end

plot(t, y)
ylabel('normalised')
xlabel 'time (s)'
title('chain output');
drawnow

if nargout == 0
    clear y
end
end
