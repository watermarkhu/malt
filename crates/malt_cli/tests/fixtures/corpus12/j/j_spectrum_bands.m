function bands = spectrum_bands(signal, fs, edges)
%SPECTRUM_BANDS Power in frequency bands of a real signal.
%   bands = SPECTRUM_BANDS(signal, fs, edges) returns the power per band.

if ~isvector(signal)
    error('spectrum:shape', 'signal must be a vector');
end
n = numel(signal);
win = hann(n);
windowed = signal(:) .* win;
S = fft(windowed);
half = floor(n / 2) + 1;
P = (abs(S(1:half)) / n).^2;
P(2:end-1) = 2 * P(2:end-1);
f = (0:half-1)' * fs / n;

nBands = numel(edges) - 1;
bands = zeros(1, nBands);
for b = 1:nBands
    inBand = f >= edges(b) & f < edges(b + 1);
    bands(b) = sum(P(inBand));
end

[~, peakIdx] = max(P(2:end));
peakFreq = f(peakIdx + 1);
peakPower = P(peakIdx + 1);
fprintf('peak %.1f Hz (%.3g)\n', peakFreq, peakPower);

db = 10 * log10(bands + eps);
loud = find(db > -20);
if numel(loud) > 0
    fprintf('%d loud bands\n', numel(loud));
end
normalized = bands / sum(bands);
bands(bands < 1e-12) = 0;
end
