%% Windowed FFT spectrum of a noisy multi-tone signal
% Builds a synthetic signal, estimates its spectrum with a Hann window
% and finds the dominant peaks with a simple local-maximum search.

clear; close all;

%% Signal definition
fs = 1000;                      % sampling rate [Hz]
T = 2;                          % duration [s]
t = (0:1/fs:T - 1/fs)';
freqs = [50, 120, 245];         % tone frequencies [Hz]
amps = [1.0, 0.5, 0.2];         % tone amplitudes
rng(42);                        % reproducible noise

x = zeros(size(t));
for k = 1:numel(freqs)
    x = x + amps(k) * sin(2*pi*freqs(k)*t);
end
x = x + 0.3 * randn(size(t));

%% Windowed, averaged FFT (Welch-style, written out by hand)
nWin = 512;
overlap = 256;
w = hann(nWin, 'periodic');
nfft = 2^nextpow2(4 * nWin);

starts = 1:(nWin - overlap):(numel(x) - nWin + 1);
nSeg = numel(starts);
P = zeros(nfft/2 + 1, 1);
for s = 1:nSeg
    seg = x(starts(s):starts(s) + nWin - 1) .* w;
    X = fft(seg, nfft);
    P = P + abs(X(1:nfft/2 + 1)).^2;
end

% Scale to a one-sided power spectral density
P = P / nSeg / (fs * sum(w.^2));
P(2:end-1) = 2 * P(2:end-1);
f = (0:nfft/2)' * fs / nfft;

%% Manual peak finding: local maxima within 25 dB of the highest bin
dB = 10 * log10(P + eps);
threshold = max(dB) - 25;
peakIdx = [];
for k = 2:numel(dB) - 1
    isLocalMax = dB(k) > dB(k-1) && dB(k) >= dB(k+1);
    if isLocalMax && dB(k) > threshold
        peakIdx(end+1) = k; %#ok<SAGROW>
    end
end
[~, order] = sort(dB(peakIdx), 'descend');
peakIdx = peakIdx(order);

%% Report
fprintf('\nDominant peaks (threshold %.1f dB):\n', threshold);
fprintf('%10s %12s\n', 'Freq [Hz]', 'Power [dB]');
for k = 1:min(5, numel(peakIdx))
    i = peakIdx(k);
    fprintf('%10.1f %12.1f\n', f(i), dB(i));
end

%% Plots
figure('Color', 'w');

subplot(2, 1, 1);
plot(t(1:300), x(1:300));
xlabel('Time [s]');
ylabel('x(t)');
title('Signal (first 300 ms)');
grid on;

subplot(2, 1, 2);
semilogy(f, P, 'LineWidth', 1);
hold on;
semilogy(f(peakIdx), P(peakIdx), 'ro', 'MarkerFaceColor', 'r');
hold off;
xlim([0 400]);
xlabel('Frequency [Hz]');
ylabel('PSD [V^2/Hz]');
title('Windowed spectrum');
grid on;
