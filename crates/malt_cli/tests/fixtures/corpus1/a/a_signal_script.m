%% Signal filtering demo
% Generate a noisy test signal, filter it, and report some statistics.
% Written for a quick look at a low-pass Butterworth filter.

clear
close all
clc

%% Parameters
fs = 1000;              % sampling frequency [Hz]
duration = 2;           % signal length [s]
f_signal = 12;          % tone frequency [Hz]
f_cutoff = 40;          % low-pass cutoff [Hz]
noise_level = 0.3;      % standard deviation of added noise
filter_order = 4;

rng(42)                 % reproducible noise

%% Generate signal
t = (0:1/fs:duration - 1/fs)';
clean = sin(2*pi*f_signal*t) + 0.5*sin(2*pi*3*f_signal*t);
noise = noise_level * randn(size(t));
noisy = clean + noise;

%% Design filter
Wn = f_cutoff / (fs/2);
[b, a] = butter(filter_order, Wn, 'low');
filtered = filtfilt(b, a, noisy);

%% Compute statistics
rmse_before = sqrt(mean((noisy - clean).^2));
rmse_after  = sqrt(mean((filtered - clean).^2));
improvement = 100 * (1 - rmse_after / rmse_before);

fprintf('RMSE before filtering: %.4f\n', rmse_before)
fprintf('RMSE after filtering:  %.4f\n', rmse_after)
fprintf('Improvement:           %.1f %%\n', improvement)

%% Spectrum
N = numel(t);
freqs = (0:N-1)' * fs / N;
Y = fft(filtered);
P = abs(Y(1:floor(N/2)+1)) / N;
[~, idx] = max(P(2:end));
dominant = freqs(idx + 1);
fprintf('Dominant frequency: %.1f Hz\n', dominant)

%% Cutoff sweep (for the notebook)
cutoffs = 20:10:80;
rmse_sweep = [];
for fc = cutoffs
    [bb, aa] = butter(filter_order, fc / (fs/2), 'low');
    yy = filtfilt(bb, aa, noisy);
    rmse_sweep = [rmse_sweep sqrt(mean((yy - clean).^2))];
end
[~, best] = min(rmse_sweep);
fprintf('Best cutoff: %d Hz\n', cutoffs(best))

%% Plot
figure('Name', 'Filter demo', 'Color', 'w')

subplot(2, 1, 1)
plot(t, noisy, 'Color', [0.7 0.7 0.7]), hold on
plot(t, clean, 'k', 'LineWidth', 1.5)
plot(t, filtered, 'r', 'LineWidth', 1)
hold off
xlabel('Time [s]')
ylabel('Amplitude')
legend('Noisy', 'Clean', 'Filtered', 'Location', 'northeast')
title('Time domain')
grid on

subplot(2, 1, 2)
semilogy(freqs(1:numel(P)), P)
xlim([0 100])
xlabel('Frequency [Hz]')
ylabel('|X(f)|')
title('Amplitude spectrum')
grid on

%% Save results
results = struct('rmse_before', rmse_before, ...
                 'rmse_after', rmse_after, ...
                 'dominant_hz', dominant);
save('filter_results.mat', 'results')
disp('Done.')
