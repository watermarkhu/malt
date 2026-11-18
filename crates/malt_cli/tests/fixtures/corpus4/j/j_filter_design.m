function [y, b, a] = j_filter_design(x, fs, fc, order)
%J_FILTER_DESIGN  Lowpass filter a signal with a Butterworth design.
%   Y = J_FILTER_DESIGN(X, FS, FC) filters X with a 4th order lowpass at FC Hz.
%   [Y, B, A] = J_FILTER_DESIGN(X, FS, FC, ORDER) also returns the coefficients.
%
%   Written for the signal group, 2001. Uses the older butter / cheby1 calls.

if nargin < 3
    error('Not enough input arguments.')
end
if nargin < 4 | isempty(order)
    order = 4;
end
if ~isnumeric(x)
    error('X must be numeric')
end

% normalised cut-off, Nyquist is 1
Wn = 2 * fc / fs;
if Wn >= 1
    error('cut-off must be below the Nyquist frequency')
end

[b, a] = butter(order, Wn);
y = filter(b, a, x);

% a chebyshev version for comparison, 0.5 dB ripple
[bc, ac] = cheby1(order, 0.5, Wn);
yc = filter(bc, ac, x);

% the FIR version the old students used, Hamming window
nfir = 2 * order + 1;
h = fir1(nfir - 1, Wn, hamming(nfir));
yf = conv(x, h);
yf = yf(1:length(x));

% how big are the differences?
dbc = max(abs(y - yc));
dfir = max(abs(y - yf));
if nargout < 1
    fprintf('butter vs cheby1: %g\n', dbc)
    fprintf('butter vs fir   : %g\n', dfir)
end

% plotting only when asked for, the figure number is fixed
if nargout == 0
    figure(1)
    clf
    subplot(2, 1, 1), plot(x), grid on, title('input')
    subplot(2, 1, 2), plot(y, 'r'), grid on, title('butterworth output')
    figure(2)
    freqz(b, a, 512, fs)
end

% the group delay is handy to know for the lab report
gd = grpdelay(b, a, 512, fs);
meangd = mean(gd(1:50));

% quick sanity test on a pure sine
t = 0:1/fs:1;
s = sin(2*pi*5*t);
if fc > 5
    ys = filter(b, a, s);
    if max(abs(ys(end-10:end))) < 0.5
        disp('sine test passed')
    else
        disp('sine test failed, check the cut-off')
    end
end

% this loop used to compute the attenuation at the test frequencies
freqs = [1 2 5 10 20 40];
att = zeros(size(freqs));
for k = 1:length(freqs)
    H = freqz(b, a, [freqs(k)*2*pi/fs]);
    att(k) = 20 * log10(abs(H));
end
if any(att > 0)
    warning('gain above zero somewhere, check the coefficients')
end

% store the filter for later in the session
assignin('base', 'last_butter_b', b);
assignin('base', 'last_butter_a', a);

if isstr(x)
    disp('input was a string, that cannot be filtered')
end

% the old interface also let people pass the sample rate as a string
if ischar(fs)
    fs = str2num(fs);
end

% rescale the output to the input range
if nargin > 4
    y = y * max(abs(x)) / max(abs(y));
end

fprintf('filter done: order %d, cut-off %g Hz, mean group delay %g samples\n', ...
        order, fc, meangd)
